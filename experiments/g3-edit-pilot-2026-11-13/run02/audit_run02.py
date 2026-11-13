#!/usr/bin/env python3
"""Audit run-02 schedule, prompt/response bytes, frozen artifacts, and score hashes."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPERIMENT = ROOT.parent
REPO = ROOT.parents[2]
MODEL = Path("/tmp/aic-g3-pilot-2026-11-13/Qwen3-4B-Q4_K_M.gguf")
COMPILER = Path("/tmp/aic-g3-pilot-2026-11-13/cargo-target/release/aic")
HELPER = Path("/tmp/aic-g3-pilot-run02-2026-11-13/helper-target/release/g3-parser-adapter")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def pair_seed(task_id: str, replicate: int) -> int:
    value = b"AIC-G3-run02-seed-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return int.from_bytes(hashlib.sha256(value).digest()[:4], "big")


def order_bit(task_id: str, replicate: int) -> int:
    value = b"AIC-G3-run02-order-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return hashlib.sha256(value).digest()[0] & 1


def main() -> int:
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    parent_manifest = json.loads((ROOT / "run01_manifest.json").read_text(encoding="utf-8"))
    preflight = json.loads((ROOT / "preflight.json").read_text(encoding="utf-8"))
    run_meta = json.loads((ROOT / "run_metadata.json").read_text(encoding="utf-8"))
    summary = json.loads((ROOT / "summary.json").read_text(encoding="utf-8"))
    raw_path = ROOT / "trials.jsonl"
    scored_path = ROOT / "scored.jsonl"
    raw = [json.loads(line) for line in raw_path.read_text(encoding="utf-8").splitlines() if line]
    scored = [json.loads(line) for line in scored_path.read_text(encoding="utf-8").splitlines() if line]

    checks: dict[str, bool] = {}
    checks["run_complete_120_calls"] = run_meta.get("status") == "complete" and run_meta.get("calls_recorded") == 120 and len(raw) == 120
    checks["contiguous_call_indices"] = [row.get("call_index") for row in raw] == list(range(120))
    checks["all_responses_completed"] = all(row.get("status") == "completed" and isinstance(row.get("response_text"), str) for row in raw)
    checks["no_inflight_call"] = not (ROOT / "inflight.json").exists()
    checks["preflight_context_fit"] = preflight.get("status") == "ready" and preflight.get("all_fit") is True
    checks["template_fixed"] = preflight.get("chat_template_sha256") == manifest["generation_path"]["chat_template_sha256"] and preflight.get("enable_thinking") is False
    checks["model_file_hash"] = MODEL.is_file() and file_sha256(MODEL) == manifest["model"]["sha256"]
    checks["compiler_binary_hash"] = COMPILER.is_file() and file_sha256(COMPILER) == manifest["scoring"]["compiler_binary_sha256"]
    checks["helper_binary_hash"] = HELPER.is_file() and file_sha256(HELPER) == manifest["scoring"]["helper_binary_sha256"]
    checks["scoring_reads_all_calls"] = len(scored) == 120 and summary.get("trials_read") == 120 and summary.get("scored_rows") == 120
    checks["scoring_raw_hash"] = summary.get("trials_sha256") == file_sha256(raw_path)
    checks["scoring_rows_hash"] = summary.get("scored_sha256") == file_sha256(scored_path)
    checks["summary_compiler_hash"] = summary.get("compiler_sha256") == manifest["scoring"]["compiler_binary_sha256"]
    checks["summary_helper_hash"] = summary.get("helper_sha256") == manifest["scoring"]["helper_binary_sha256"]
    checks["preregistration_hash"] = file_sha256(ROOT / "preregistration.md") == manifest["preregistration_sha256"]
    checks["runner_hash"] = file_sha256(ROOT / "run_pilot.py") == manifest["runner_sha256"]
    checks["frozen_repository_file_hashes"] = all(
        (REPO / relative).is_file() and file_sha256(REPO / relative) == expected
        for relative, expected in manifest["repository_files_sha256"].items()
    )
    checks["production_source_hashes"] = all(
        (REPO / relative).is_file() and file_sha256(REPO / relative) == expected
        for relative, expected in manifest["scoring"]["compiler_source_sha256"].items()
    )

    prompt_by = {(entry["task_id"], entry["format"]): entry for entry in parent_manifest["trial_prompts"]}
    expected_schedule = []
    for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
        for replicate in range(10):
            bit = order_bit(task_id, replicate)
            pair = ("aic", "json") if bit == 0 else ("json", "aic")
            expected_schedule.extend((task_id, replicate, form, pair, bit, pair_seed(task_id, replicate)) for form in pair)
    checks["schedule_and_pair_randomization"] = len(raw) == len(expected_schedule) and all(
        (row.get("task_id"), row.get("replicate"), row.get("format"), tuple(row.get("pair_order", [])), row.get("order_bit"), row.get("pair_seed")) == slot
        for row, slot in zip(raw, expected_schedule)
    )
    prompts_match = True
    response_bytes_match = True
    trial_scores_match = len(scored) == len(raw)
    for index, row in enumerate(raw):
        prompt_entry = prompt_by[(row["task_id"], row["format"])]
        prompt_bytes = (EXPERIMENT / prompt_entry["path"]).read_bytes()
        prompts_match &= (
            sha256(prompt_bytes) == prompt_entry["sha256"] == row.get("prompt_sha256")
            and row.get("user_prompt_utf8_hex") == prompt_bytes.hex()
            and row.get("user_prompt") == prompt_bytes.decode("utf-8")
        )
        response = row.get("response_text")
        response_bytes_match &= isinstance(response, str) and row.get("response_utf8_hex") == response.encode("utf-8").hex()
        if index < len(scored):
            score = scored[index]
            trial_scores_match &= all(score.get(key) == row.get(key) for key in ("call_index", "task_id", "replicate", "format", "status", "pair_seed", "order_bit"))
    checks["prompt_packet_bytes_match"] = bool(prompts_match)
    checks["response_utf8_bytes_match"] = bool(response_bytes_match)
    checks["scoring_row_identity_matches_raw"] = bool(trial_scores_match)
    checks["no_think_wrapper_in_generated_content"] = not any(row["response_text"].startswith("<think>\n\n</think>\n\n") for row in raw)

    format_summary = {}
    for form in ("aic", "json"):
        rows = [row for row in scored if row.get("format") == form]
        format_summary[form] = {
            "calls": len(rows),
            "parse_accepted": sum(bool(row.get("parse_accepted")) for row in rows),
            "validator_accepted": sum(bool(row.get("validator_accepted")) for row in rows),
            "edit_pass": sum(bool(row.get("edit_pass")) for row in rows),
            "object_checks": sum(row.get("object_check") is not None for row in rows),
            "binary_matches_candidate_oracle": sum(bool(row.get("object_check") and row["object_check"].get("binary_matches_candidate_oracle")) for row in rows),
            "binary_matches_task_oracle": sum(bool(row.get("object_check") and row["object_check"].get("binary_matches_task_expected")) for row in rows),
            "pilot_success": sum(bool(row.get("pilot_success")) for row in rows),
        }
    checks["all_compiled_outputs_match_candidate_oracle"] = all(
        item["object_checks"] == item["parse_accepted"] == item["validator_accepted"] == item["binary_matches_candidate_oracle"]
        for item in format_summary.values()
    )
    checks["format_counts_reconcile"] = all(format_summary[form]["calls"] == 60 for form in ("aic", "json"))

    audit = {
        "status": "PASS" if all(checks.values()) else "FAIL",
        "checks": checks,
        "format_summary": format_summary,
        "raw_trials_sha256": file_sha256(raw_path),
        "scored_rows_sha256": file_sha256(scored_path),
        "run_metadata_sha256": file_sha256(ROOT / "run_metadata.json"),
        "manifest_sha256": file_sha256(ROOT / "manifest.json"),
        "preregistration_sha256": file_sha256(ROOT / "preregistration.md"),
        "model_sha256": file_sha256(MODEL),
        "compiler_binary_sha256": file_sha256(COMPILER),
        "helper_binary_sha256": file_sha256(HELPER),
        "known_scope_limit": "Six synthetic single-block edit tasks under one local pinned model; exploratory evidence only.",
    }
    (ROOT / "audit.json").write_text(json.dumps(audit, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(audit, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if audit["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
