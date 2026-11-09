# AI-compiler

**A new compiler project that translates AI-native, machine-operable text into machine instructions compatible with existing computing standards.**

> **Status (2026-11-09): the owner adopted the initial R2 scope; the R3 vertical slice is wired and local R4 QC has been observed.** The current instruction forms and the gABI-common clauses in ELF Object File Format 4.2 have been checked against primary sources. The minimum CPU ISA feature baseline and ELF object-format reference edition remain undecided. An open map of language and representation families is seeded, while individual comparisons remain in progress. R1, target QA, and beta publication are incomplete and unapproved.

[日本語正本](README.md) · [Specification (Japanese)](docs/01_統合仕様書.md) · [Roadmap (Japanese)](docs/02_工程表.md) · [Codex instructions (Japanese)](docs/03_Codex施工指示書.md) · [Agent rules](AGENTS.md)

## Goal

AI should be able to generate and manipulate a purpose-designed textual representation, which a new compiler converts into machine instructions and relevant executable artifacts while preserving semantics and respecting explicitly selected ISA, ABI, OS-interface, and object-format requirements.

**Human readability is not a mandatory property of the AI-native input representation.** The semantics must nevertheless remain unambiguous and mechanically checkable.

Conventional high-level programming languages are not required as **intermediate representations of the programs being compiled**. The owner selected Rust 2024 + Cargo for the compiler and C++20 for the independent test harness.

## Adopted initial implementation scope

- Input: AIC/0.2, one `u64` argument, one `u64` result from `aic_main`. The initial operations are wrapping `u64` add/subtract, equality, CFG branches, Phi, and return.
- Target: Linux x86-64 / System V AMD64 ABI / ELF64 ET_REL only. GNU as creates relocatable objects; linking happens only in the test harness.
- Unsupported: memory, pointers, I/O, syscalls, globals, multiple functions, calls, exceptions, atomics/threads, floating point/vectors, and nondeterminism.
- The initial backend uses stack slots and does not optimize for performance. Unsupported targets and operations are rejected.

This adoption establishes the initial implementation scope. It does not declare standards conformance complete or approve beta publication. The current instruction forms and common gABI clauses received a limited source review; target conditions and full-clause conformance remain open. Test results are not generalized to every CPU or environment.

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

The Rust compiler connects AIC/0.2 parsing, type/CFG/SSA/Phi validation, a reference interpreter, x86-64 assembly generation, and GNU as object creation. At runtime it checks that `as --version` identifies GNU assembler, then a C++20 harness links and executes the object. Tests compare every `u16` input and `u64` boundary cases for a branch fixture, inputs 0..255 for a loop/Phi-cycle fixture, a loop with three header Phis and two backedges that swaps a Phi pair on one latch, a generated 24-stage CFG with three-predecessor Phi merges, and a long stack frame against both the reference interpreter and independent expected-value models. They also inspect ELF headers, symbols, relocations, disassembly, ABI RSP/RBP and callee-saved state, and byte-for-byte repeat output. A negative test confirms that the ABI probe rejects a function that clobbers RBP. Parser mutation/truncation cases are checked for panics. Objects are generated and checked in a private temporary directory before installation at the requested output path. These are local host QC results, not full ISA/gABI review or a formal semantic-preservation proof.

The debug suite currently has 25 Rust unit tests, 3 CLI tests, and 7 end-to-end tests. Local checks:

~~~sh
export PATH="${HOME}/.cargo/bin:${PATH}"
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
~~~

End-to-end tests require GNU as/binutils and g++ 14 or later. The Rust toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). To compile an object:

~~~sh
cargo run -- compile --target x86_64-linux-gnu examples/branch_phi.aic -o branch_phi.o
~~~

Work products: [compiler and standards research (Japanese)](docs/research/01_既存技術・規格調査.md) · [open language and representation map (Japanese)](docs/research/02_既存言語・表現参照地図.md) · [first syntax and validation comparison (Japanese)](docs/research/03_入力表現・構文・検証境界比較.md) · [structured editing and schema evolution comparison (Japanese)](docs/research/04_構造化編集・版進化比較.md) · [control flow, effects, and state model comparison (Japanese)](docs/research/05_制御フロー・効果・状態モデル比較.md) · [integer types and numeric boundary semantics comparison (Japanese)](docs/research/06_整数型・数値境界意味比較.md) · [adopted initial compilation contract and open questions (Japanese)](docs/proposals/01_初期変換契約案.md)

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
