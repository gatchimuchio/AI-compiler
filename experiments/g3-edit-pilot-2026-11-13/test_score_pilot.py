#!/usr/bin/env python3
"""Offline fixtures for the isolated G3 adapter and scorer; no model calls."""

import importlib.util
import json
import unittest
from pathlib import Path


HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("score_pilot", HERE / "score_pilot.py")
score = importlib.util.module_from_spec(spec)
spec.loader.exec_module(score)


class CandidateAdapterTests(unittest.TestCase):
    def test_valid_json_normalizes_to_aic(self):
        candidate = {
            "format": "AIC-J/0.1",
            "function": {
                "name": "aic_main",
                "params": [{"id": "x", "type": "u64"}],
                "result": "u64",
                "blocks": [{
                    "id": "entry",
                    "ops": [
                        {"id": "one", "op": "const.u64", "value": "1"},
                        {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]},
                    ],
                    "term": {"op": "ret.u64", "value": "sum"},
                }],
            },
        }
        ops, source = score.validate_json_profile(candidate)
        self.assertEqual(ops[-1], {"op": "ret.u64", "value": "sum"})
        self.assertIn("%sum = add.wrap.u64 %x, %one", source)

    def test_duplicate_json_key_rejected(self):
        ok, parsed, error = score.parse_candidate_json('{"format":"AIC-J/0.1","format":"AIC-J/0.1"}')
        self.assertFalse(ok)
        self.assertIsNone(parsed)
        self.assertIn("duplicate", error)

    def test_unknown_schema_key_rejected(self):
        with self.assertRaises(score.CandidateError):
            score.validate_json_profile({"format": "AIC-J/0.1", "function": {}, "extra": 1})

    def test_forward_reference_rejected(self):
        candidate = {
            "format": "AIC-J/0.1",
            "function": {
                "name": "aic_main", "params": [{"id": "x", "type": "u64"}], "result": "u64",
                "blocks": [{"id": "entry", "ops": [
                    {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]},
                    {"id": "one", "op": "const.u64", "value": "1"},
                ], "term": {"op": "ret.u64", "value": "sum"}}],
            },
        }
        with self.assertRaisesRegex(score.CandidateError, "forward"):
            score.validate_json_profile(candidate)

    def test_wraparound_oracle(self):
        ops = [
            {"id": "one", "op": "const.u64", "value": "1"},
            {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]},
            {"op": "ret.u64", "value": "sum"},
        ]
        self.assertEqual(score.independent_oracle(ops, 2**64 - 1), 0)

    def test_allowed_target_fields_are_excluded_from_unrequested_changes(self):
        expected = [
            {"id": "one", "op": "const.u64", "value": "17"},
            {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]},
            {"op": "ret.u64", "value": "sum"},
        ]
        changed = [dict(expected[0], value="18"), dict(expected[1], op="sub.wrap.u64"), expected[2]]
        self.assertEqual(score.unrequested_changes(changed, expected, ["i0.value"]), 1)


if __name__ == "__main__":
    unittest.main()
