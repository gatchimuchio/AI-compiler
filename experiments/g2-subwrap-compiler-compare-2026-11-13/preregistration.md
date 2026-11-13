# G2 comparison plan: modular subtraction

- Record ID: `G2-COMPARE-2026-11-13-02`
- Owner task: continue AICompiler's G1/G2 work under the uploaded 2026-11-13 three-document set.
- Subject: AICompiler's route from an AI-edited operation string to an ET_REL object and consumer result, compared with existing C and Rust compiler routes for the same operation.
- Task: given `x: u64`, return `x - 1` modulo `2^64`; edit the existing AIC program by changing only the operation from `add.wrap.u64` to `sub.wrap.u64`.
- Selection provenance: use existing G3 run 02 call 21 (`E2`, AIC, replicate 0, pair seed `4067340202`) because it already contains a completed identified-model response. Selection followed the known pilot outcome. No inference will be repeated, and the comparison will be reported as a post-hoc illustrative case rather than an unbiased benchmark.
- Inputs: use the exact 22-value E2 run 02 vector, in recorded order. Expected output is `(x - 1) mod 2^64` for each input.
- Routes: (A) pinned project compiler `aic compile --target x86_64-linux-gnu` on the byte-identical Qwen3 AIC response; GNU as settings are those of the pinned project compiler. (B) GCC 14.2.0 C11 source using target `uint64_t` and `-std=c11 -O2 -march=x86-64 -mtune=generic -fno-pic -fno-pie -fno-stack-protector`. (C) rustc 1.99.1 Rust 2024 source using `u64::wrapping_sub`, `extern "C"`, and `--target x86_64-unknown-linux-gnu --crate-type=lib --emit=obj -C opt-level=2 -C target-cpu=x86-64 -C relocation-model=static -C panic=abort`.
- Consumer: the same checked-in `tests/harness.cpp` and `tests/abi_probe.S`, compiled separately against each object with GNU g++ 14.2.0 and the same C++20 flags. The consumer must exit 0 and emit one decimal result per input. Each execution is capped at 30 seconds.
- Evidence: save all three source files, source and tool hashes, command outputs, ET_REL headers/sections, function disassembly and byte count, object and consumer hashes/sizes, consumer output bytes/hash, and oracle comparison. No throughput claim or ranking will be made because the AIC backend is not optimized like GCC/rustc and no timing protocol is planned.
- Stop conditions: stop a route if the expected compiler, input, object type, consumer exit, or output shape is absent or differs. Preserve the failure; do not change the source, flags, input vector, or oracle after observing output.
- Scope: one synthetic edit task, three compiler paths, one Linux x86-64 environment, and 22 values. It does not establish G2 completion, long-term operational history, performance superiority, full ABI/ELF QA, or adoption of a new AI format.

The frozen source bytes and input vector are captured in `inputs/frozen_case.json` before invoking GCC or rustc. The existing AIC trial and its original G3 score remain immutable.
