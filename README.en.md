# AI-compiler

**A new compiler project that translates AI-native, machine-operable text into machine instructions compatible with existing computing standards.**

> **Status (2026-11-11): the owner adopted the initial R2 scope and the target profile's CPU floor and ELF reference edition; the R3 vertical slice is wired and local R4 QC has been observed.** The Linux x86-64 floor is AMD64 psABI 1.0 `baseline`, and the gABI-common object reference is stable ELF Object File Format 4.2. The CLI target `x86_64-linux-gnu` is a project-specific alias for that profile. The broader R1 survey, target QA, and beta publication remain incomplete; beta publication is unapproved.

[日本語正本](README.md) · [Specification (Japanese)](docs/01_統合仕様書.md) · [Roadmap (Japanese)](docs/02_工程表.md) · [Codex instructions (Japanese)](docs/03_Codex施工指示書.md) · [Agent rules](AGENTS.md)

## Goal

AI should be able to generate and manipulate a purpose-designed textual representation, which a new compiler converts into machine instructions and relevant executable artifacts while preserving semantics and respecting explicitly selected ISA, ABI, OS-interface, and object-format requirements.

**Human readability is not a mandatory property of the AI-native input representation.** The semantics must nevertheless remain unambiguous and mechanically checkable.

Conventional high-level programming languages are not required as **intermediate representations of the programs being compiled**. The owner selected Rust 2024 + Cargo for the compiler and C++20 for the independent test harness.

## Adopted initial implementation scope

- Input: AIC/0.2, one `u64` argument, one `u64` result from `aic_main`. The initial operations are wrapping `u64` add/subtract, equality, CFG branches, Phi, and return.
- Target: Linux x86-64 / AMD64 psABI 1.0 `baseline` / System V AMD64 ABI / ELF64 ET_REL only. The gABI-common object reference is ELF Object File Format 4.2. GNU as receives `--64 -march=generic64`; linking happens only in the test harness.
- Unsupported: memory, pointers, I/O, syscalls, globals, multiple functions, calls, exceptions, atomics/threads, floating point/vectors, and nondeterminism.
- The initial backend uses stack slots and does not optimize for performance. Unsupported targets and operations are rejected.

This adoption establishes the initial implementation scope and target profile: AMD64 psABI 1.0 `baseline` and ELF Object File Format 4.2 for common gABI object rules. It does not declare standards conformance complete or approve beta publication. The current instruction forms and common gABI clauses received a limited source review; full-clause conformance remains open. Test results are not generalized to every CPU or environment.

## Scope and boundaries

- Keep a broad, open reference map of existing languages, IRs, DSLs, data/schema formats, and formal descriptions. Resource limits determine review depth and order; deferred branches retain their reason and reopening conditions, and remain distinct from rejected or unadopted candidates.
- Existing compiler implementations are one part of this reference space; LLVM/GCC/CompCert do not stand in for all existing languages or representations. A seed list does not claim every language or version has been reviewed.
- Design a new AI-native input contract; do not assume an existing programming language or compiler IR is canonical.
- Implement correct translation for explicitly supported targets and operations; reject unsupported cases.
- Preserve deterministic translation, semantic meaning, traceable conformance evidence, and defined responsibility boundaries.
- The project does **not** redesign machine language, the CPU ISA, physical memory organization, or the earlier `coding-operation-ir-compiler` / `machine-realization-core` projects.
- A limited initial target scope does **not** justify relaxing correctness requirements inside that declared scope.

## Evidence and release status

The specification, implementation, source-code presence, verified behavior, real-hardware behavior, and product release are separate states. The initial platform, input contract, and implementation technologies have been selected. Formal conformance evidence, QA thresholds, and release acceptance criteria remain open.

## Current implementation and checks

The Rust compiler connects AIC/0.2 parsing, type/CFG/SSA/Phi validation, a reference interpreter, x86-64 assembly generation, and GNU as object creation. At runtime it checks that `as --version` identifies GNU assembler, then a C++20 harness links and executes the object. Tests compare every `u16` input and `u64` boundary cases for a branch fixture, inputs 0..255 for a loop/Phi-cycle fixture, a loop with three header Phis and two backedges that swaps a Phi pair on one latch, a generated 24-stage CFG with three-predecessor Phi merges, and a long stack frame against both the reference interpreter and independent expected-value models. They also inspect ELF headers, symbols, relocations, disassembly, ABI RSP/RBP, callee-saved GPRs, DF, x87 control word, and MXCSR controls, and byte-for-byte repeat output. The ABI probe uses non-default rounding modes and rejects RBP, DF, x87 control-word, and MXCSR-control violations while accepting caller-saved MXCSR status changes. The C++ consumer independently checks that the probe restores its caller's control state. Parser mutation/truncation cases are checked for panics. Objects are generated in a private temporary directory and checked for headers, section ranges, string/symbol references, and the defined `aic_main` function before installation. Extended numbering and compressed interpreted tables are currently unsupported. The current backend also rejects REL/RELA sections, dynamic tables, non-null undefined symbols, and common symbols before installation because it does not emit them. These output limits do not establish general ELF invalidity, machine-code purity, or absence of every relocation extension. These are local host QC results, not full ISA/gABI review or a formal semantic-preservation proof.

Current implementation capacities are 1 MiB of source, 256 blocks, 16,384 instructions, 65,536 Phi incoming records, 63 bytes per public-IR identifier, and 8 MiB of observed assembler object bytes. Parameters and terminators do not count as instructions. Excess capacity is rejected with `E_LIMIT`; the CLI reads at most the source cap plus one byte and can reject excess input before EOF. These new implementation limits narrow processing capacity; they do not make larger sources semantically invalid or establish total memory, time, or beta resource guarantees. ELF inspection also uses the final-NUL invariant and a fixed-name comparison to avoid rescanning long strings for every reference. See [contract §2.5 (Japanese)](docs/proposals/01_初期変換契約案.md).

On 2026-11-11, both debug and release suites pass 45 Rust unit tests, 5 CLI tests, and 10 end-to-end tests. The public Rust emitter revalidates the current IR and checks the supplied metadata. Tests reject empty entry Phis and mismatched metadata, and compare 48 self-edge Phi variants with three-cycle, self-copy, and fanout behavior against independent gold on 263 inputs each. Another 43,248 ordinary/Phi-edge dominance decisions on four-block CFGs match an independent path-search oracle. Native fixtures also cover a two-entry cycle and alternating latch maps. These are finite QC populations. Local checks:

~~~sh
export PATH="${HOME}/.cargo/bin:${PATH}"
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
~~~

End-to-end tests require GNU as/binutils and g++ 14 or later. CLI stream and end-to-end tests also require GNU coreutils `timeout`. The common native runner has a 30-second local test deadline; this does not guarantee AIC termination or product performance. The Rust toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). To compile an object:

~~~sh
cargo run -- compile --target x86_64-linux-gnu examples/branch_phi.aic -o branch_phi.o
~~~

Work products: [compiler and standards research (Japanese)](docs/research/01_既存技術・規格調査.md) · [open language and representation map (Japanese)](docs/research/02_既存言語・表現参照地図.md) · [first syntax and validation comparison (Japanese)](docs/research/03_入力表現・構文・検証境界比較.md) · [structured editing and schema evolution comparison (Japanese)](docs/research/04_構造化編集・版進化比較.md) · [control flow, effects, and state model comparison (Japanese)](docs/research/05_制御フロー・効果・状態モデル比較.md) · [integer types and numeric boundary semantics comparison (Japanese)](docs/research/06_整数型・数値境界意味比較.md) · [memory references, lifetime, and access boundaries comparison (Japanese)](docs/research/07_メモリ参照・寿命・アクセス境界比較.md) · [diagnostic obligations, locations, and recovery comparison (Japanese)](docs/research/08_診断・位置情報・エラー回復比較.md) · [adopted initial compilation contract and open questions (Japanese)](docs/proposals/01_初期変換契約案.md)

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
