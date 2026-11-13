# G3 pilot run 02 preregistration

- **Status:** frozen before any E1–E6 generation call for run 02.
- **Research subject:** AICompiler's candidate operation-symbol formats for a fixed single-block local edit task.
- **Purpose:** determine whether the initial zero-parse result was caused by a missing Qwen3 non-thinking chat-template setting, then obtain a valid exploratory paired comparison under that corrected, equal-for-both-formats condition.
- **Evidence boundary:** run 02 is a separate exploratory replication. Do not combine its rows with run 01, infer a general winner, adopt AIC-J/0.1, define product QA thresholds, or claim overall format optimality.

## Diagnostic basis

Run 01 is preserved as originally scored: 119 complete responses, one unknown JSON slot, and strict parser acceptance of 0/60 AIC and 0/59 JSON. Every complete response began with an empty `<think>...</think>` wrapper. The pinned `llama-cpp-python==0.3.16` `Jinja2ChatFormatter.__call__` accepts arbitrary keyword arguments but does not forward them to Jinja rendering, and its registered chat completion handler invokes the formatter without the Qwen3 `enable_thinking` variable. The original `/no_think` system text therefore did not select Qwen3's non-thinking template mode.

Before this preregistration, a single **non-task** feasibility probe used the same model and template with `enable_thinking=false` explicitly bound in the formatter environment. It returned exactly `OK`, with no generated empty-think wrapper. The probe prompt was not an E1–E6 task, it was not scored, and it is not part of the format-comparison sample. The probe script and full result are recorded in the parent `diagnostics/` directory.

## Fixed comparison

- **Control:** adopted AIC/0.2 single-block profile.
- **Candidate:** unadopted, pilot-only AIC-J/0.1 profile defined by the parent protocol and exact prompt packets.
- **Task bank:** the parent's fixed E1–E6 packets, unchanged and SHA-256 checked against the run 01 manifest.
- **Oracle and compiler:** the parent's fixed inputs and independent Python modulo-2^64 oracle, isolated JSON adapter, production `ai_compiler::parse`/`validate`, production CLI, Linux x86-64/System V ABI/ELF64 ET_REL profile, and C++ consumer. Production source and compiler-binary hashes are recorded in `manifest.json`; no production source change is part of this run.
- **Primary outcome:** the parent strict-scoring definition: exact syntax/profile parse, common validation, requested edit, and agreement between the compiled object's 22 fixed-input outputs and the independent task oracle. Report each gate separately.
- **Prompt handling:** system message remains `/no_think`; user packet bytes remain unchanged. The GGUF embedded chat template is rendered with the boolean `enable_thinking=false` for every AIC and JSON call. The empty wrapper is part of the formatted model input and is not part of generated output. Do not strip wrappers, add repair turns, or change prompts after generation begins.
- **Model:** `Qwen/Qwen3-4B-GGUF`, pinned revision `bc640142c66e1fdd12af0bd68f40445458f3869b`, file `Qwen3-4B-Q4_K_M.gguf`, 2,497,280,256 bytes, SHA-256 `7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5`; Apache-2.0 per pinned model card. Use the existing local file; no additional model download or remote inference.
- **Runtime and sampling:** `llama-cpp-python==0.3.16`; local CPU, 5 threads, context 4096; temperature 0.7, top_p 0.8, top_k 20, min_p 0, presence penalty 1.5, frequency penalty 0, repeat penalty 1.0, mirostat off, maximum output 1024 tokens, non-streaming, tools/web disabled. Same sampling and seed within each pair.
- **Schedule:** 6 tasks × 10 paired replicates = 60 pairs / 120 first responses. New seed/order prefixes `AIC-G3-run02-seed-v1` and `AIC-G3-run02-order-v1`; per-task randomized pair order, no repair and no retry. One external interruption leaves that slot unknown; no second call for that slot.
- **Budget:** maximum 120 model calls and 4 hours wall clock for this separately registered run; five local CPU threads; zero inference charge; zero additional model-download bytes. Stop at the bound and report missing calls as unknown/NOT_RUN.
- **Artifacts:** store all raw prompts, response text/raw API objects, per-call metadata, template hash, results, scoring outputs, runner, and protocol in this repository. Keep model weights and generated objects/consumer/build artifacts outside Git under `/tmp/aic-g3-pilot-run02-2026-11-13/` (with the already downloaded model reused from `/tmp/aic-g3-pilot-2026-11-13/`). Do not upload model weights.

## Freeze record

`manifest.json` fixes the exact runner/scorer/oracle/input/prompt/model/template/compiler identities and all run parameters. `preflight.json` records the rendered prompt token counts and fit check before the first E1–E6 call. All run outputs are written only to this directory and the distinct external run 02 artifact root. Run 01 files are not rewritten.

The first runner setup attempt stopped before model loading or any E1–E6 call because the prompt-hash lookup used the wrong manifest nesting. That `KeyError` is recorded in `setup_attempts.jsonl`; the lookup and runner hash were corrected before generation. No task prompt or response was produced in that attempt.
