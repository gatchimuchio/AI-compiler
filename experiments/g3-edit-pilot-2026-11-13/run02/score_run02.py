#!/usr/bin/env python3
"""Run the unchanged strict pilot scorer into the run-02 directory."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPERIMENT = ROOT.parent
REPO = ROOT.parents[2]
sys.path.insert(0, str(EXPERIMENT))

import score_pilot as scorer  # noqa: E402


def main() -> int:
    scorer.ROOT = ROOT
    scorer.REPO = REPO
    sys.argv = [
        str(EXPERIMENT / "score_pilot.py"),
        "--trials", str(ROOT / "trials.jsonl"),
        "--aic", "/tmp/aic-g3-pilot-2026-11-13/cargo-target/release/aic",
        "--helper-target", "/tmp/aic-g3-pilot-run02-2026-11-13/helper-target",
    ]
    return scorer.main()


if __name__ == "__main__":
    raise SystemExit(main())
