# G3 pilot run 02 — Qwen3 template-controlled replication

This is a separately preregistered exploratory follow-up to the original pilot. It uses the same six paired edit tasks, candidate/control profiles, pinned Qwen3 GGUF, and strict scoring oracle. The only generation-path change is to bind `enable_thinking=false` in the pinned GGUF chat template for both formats. It does not strip or otherwise repair generated response text.

Run 01 remains immutable and its results are not pooled with this run. The one non-task `OK` probe is recorded separately in the parent `diagnostics/` directory. The follow-up can still produce an inconclusive format comparison and does not adopt AIC-J/0.1.

The model weights and build/runtime products remain outside Git. The raw prompts, responses, call metadata, scorer output, and preregistration are stored in this directory.

Run from the repository root with the existing pinned environment and isolated external build paths:

```sh
timeout --signal=TERM --kill-after=5 4h /tmp/aic-g3-pilot-2026-11-13/venv/bin/python experiments/g3-edit-pilot-2026-11-13/run02/run_pilot.py
PATH=/home/agent/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/aic-g3-pilot-run02-2026-11-13/helper-target cargo build --release --locked --offline --manifest-path experiments/g3-edit-pilot-2026-11-13/rust-helper/Cargo.toml
/tmp/aic-g3-pilot-2026-11-13/venv/bin/python experiments/g3-edit-pilot-2026-11-13/run02/score_run02.py
```

The runner refuses a second invocation if a raw log or in-flight marker exists. A call interrupted before its response is durably recorded is unknown and must not be resent.

## Result

Run 02 completed 120/120 responses in 3,155.37 seconds, with no inference errors or unknown slots. Strict parse/validator/object/consumer counts were 50/60 for each format; all 100 parsed outputs matched the independent oracle for their actual candidate program. Exact requested-edit plus task-oracle pilot success was AIC 20/60 and JSON 19/60. The paired difference was task-dependent (JSON +9 on E4; AIC +10 on E5; zero on the other four tasks), so this exploratory sample does not select a winner or adopt AIC-J/0.1. See [the follow-up evidence report](../../docs/evidence/G3_Qwen3テンプレート補正追試_2026-11-13.md), `summary.json`, and `audit.json`.
