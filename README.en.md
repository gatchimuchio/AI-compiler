# AI-compiler

**A new compiler project that translates AI-native, machine-operable text into machine instructions compatible with existing computing standards.**

> **Status (2026-11-09): the owner adopted the initial R2 scope; the R3 vertical slice is wired and local R4 QC has been observed.** R1 remains in progress because the Intel/AMD ISA primary manuals and ELF gABI have not been reviewed. Target QA and beta publication are incomplete and unapproved.

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

This adoption establishes the initial implementation scope. It does not declare standards conformance complete or approve beta publication. The ISA primary references and ELF gABI remain unverified, and test results are not generalized to every CPU or environment.

## Scope and boundaries

- Study existing compilers and official standards as implementation evidence and reference material.
- Design a new AI-native input contract; do not assume an existing programming language or compiler IR is canonical.
- Implement correct translation for explicitly supported targets and operations; reject unsupported cases.
- Preserve deterministic translation, semantic meaning, traceable conformance evidence, and defined responsibility boundaries.
- The project does **not** redesign machine language, the CPU ISA, physical memory organization, or the earlier `coding-operation-ir-compiler` / `machine-realization-core` projects.
- A limited initial target scope does **not** justify relaxing correctness requirements inside that declared scope.

## Evidence and release status

The specification, implementation, source-code presence, verified behavior, real-hardware behavior, and product release are separate states. The initial platform, input contract, and implementation technologies have been selected. Formal conformance evidence, QA thresholds, and release acceptance criteria remain open.

## Current implementation and checks

The Rust compiler connects AIC/0.2 parsing, type/CFG/SSA/Phi validation, a reference interpreter, x86-64 assembly generation, and GNU as object creation. A C++20 harness links and executes the object. Tests compare every `u16` input and `u64` boundary cases for a branch fixture, inputs 0..255 for a loop/Phi-cycle fixture, and inspect ELF headers, symbols, relocations, disassembly, ABI register/stack state, and byte-for-byte repeat output. These are local host QC results, not full ISA/gABI review or a formal semantic-preservation proof.

The debug suite currently has 23 Rust unit tests, 3 CLI tests, and 4 end-to-end tests. Local checks:

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

Work products: [compiler and standards research (Japanese)](docs/research/01_既存技術・規格調査.md) · [adopted initial compilation contract and open questions (Japanese)](docs/proposals/01_初期変換契約案.md)

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
