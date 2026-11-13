# G3 isolated single-block edit pilot

This directory contains the preregistered AIC/0.2 vs pilot-only AIC-J/0.1 run. The candidate profile remains unadopted. The pilot does not select a winner, set a QA threshold, or change the production compiler path.

`prepare_pilot.py` materializes fixed prompt packets, the private scoring oracle, the 22 fixed inputs, and `manifest.json`. `run_pilot.py` makes one local CPU generation call per task/replicate/format, writes each result synchronously to `trials.jsonl`, and refuses to resume if that file exists. `score_pilot.py` checks strict JSON shape and duplicate keys, normalizes JSON to AIC/0.2, then uses the production Rust parser and validator through the isolated `rust-helper`. Accepted outputs are compiled by the production CLI, linked with the repository's existing ABI probe and C++ consumer, and checked against an independent Python modular-u64 evaluator.

The model weights, Python environment, compiler build, parser helper build, and generated object/consumer artifacts stay outside Git under `/tmp/aic-g3-pilot-2026-11-13/`. Raw prompt/response records, scoring output, and the experiment source are kept here. No model weight is committed.

## Interruption recovery

The first one-shot process was externally interrupted after 24 completed records. The following scheduled call, index 24 (`E2`, replicate 2, JSON), may have started but has no durable response. The recovery addendum records it as `interrupted_unknown`; it is never resent. `resume_pilot.py` continues at index 25 with the same model, prompts, sampling, seeds, randomized order, and original cumulative four-hour budget. The recovery decision was made before any trial response was scored or outcome inspected. The addendum and source hash are in `recovery_addendum.json`.

`run_pilot.py` remains the original one-shot runner and still refuses to resume an existing raw file. Only the separate, hash-pinned recovery script handles this interruption, and it durably marks each subsequent in-flight call before generation so that a later interruption cannot cause a retry.

## Result

All 120 scheduled slots are accounted for: 119 completed responses and one unknown JSON response. Strict-format parse accepted 0/60 AIC responses and 0/59 completed JSON responses. Every completed response began with an empty `<think>...</think>` wrapper, so no model output reached the validator or candidate object/consumer checks. The candidate remains unadopted; see [the evidence report](../../docs/evidence/G3_単一block局所編集pilot_2026-11-13.md), `summary.json`, and `scored.jsonl`.

## Separate follow-up (run 02)

After identifying that `llama-cpp-python==0.3.16` did not bind Qwen3's `enable_thinking` template variable, a separately preregistered run set it to `false` for both formats. Run 02 completed 120/120 responses; strict parse/validator/object/consumer passed for 50/60 per format, and all 100 objects matched the independent oracle for their actual candidate operation. Exact requested-edit plus task-oracle pilot success was AIC 20/60 and JSON 19/60, with opposing task-specific differences. Run 02 is not pooled with this historical run; it does not select a winner or adopt AIC-J/0.1. See [run 02 README](run02/README.md), [follow-up evidence](../../docs/evidence/G3_Qwen3テンプレート補正追試_2026-11-13.md), and [run 02 artifacts](run02/).

Reproduce the preflight/build/scoring commands from the repository root:

```sh
python3 experiments/g3-edit-pilot-2026-11-13/prepare_pilot.py
python3 -m unittest discover -s experiments/g3-edit-pilot-2026-11-13 -p 'test_*.py'
PATH=/home/agent/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/aic-g3-pilot-2026-11-13/cargo-target cargo build --release --locked --offline
PATH=/home/agent/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/aic-g3-pilot-2026-11-13/helper-target cargo build --release --locked --offline --manifest-path experiments/g3-edit-pilot-2026-11-13/rust-helper/Cargo.toml
```

For a new pilot, run the one-shot model trial using the isolated environment's Python and enforce the four-hour cap with GNU `timeout`. Do not rerun after a partial or completed `trials.jsonl`. For this interrupted run only, use `resume_pilot.py` under the remaining cumulative budget described by `recovery_addendum.json`.

```sh
timeout --signal=TERM --kill-after=5 4h /tmp/aic-g3-pilot-2026-11-13/venv/bin/python experiments/g3-edit-pilot-2026-11-13/run_pilot.py
```

Score after generation:

```sh
python3 experiments/g3-edit-pilot-2026-11-13/score_pilot.py
```
