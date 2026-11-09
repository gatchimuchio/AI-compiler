# AI-compiler

**A new compiler project that translates AI-native, machine-operable text into machine instructions compatible with existing computing standards.**

> **Status (2026-11-09): repository baseline complete; R1 research in progress; R2 design candidates under revision.** The ISA primary manuals and ELF gABI remain unverified, and the input and target candidates have not been adopted by the owner. No compiler implementation, build, execution, target conformance, performance gain, or product release has been demonstrated.

[日本語正本](README.md) · [Specification (Japanese)](docs/01_統合仕様書.md) · [Roadmap (Japanese)](docs/02_工程表.md) · [Codex instructions (Japanese)](docs/03_Codex施工指示書.md) · [Agent rules](AGENTS.md)

## Goal

AI should be able to generate and manipulate a purpose-designed textual representation, which a new compiler converts into machine instructions and relevant executable artifacts while preserving semantics and respecting explicitly selected ISA, ABI, OS-interface, and object-format requirements.

**Human readability is not a mandatory property of the AI-native input representation.** The semantics must nevertheless remain unambiguous and mechanically checkable.

Conventional high-level programming languages are not required as **intermediate representations of the programs being compiled**. The language used to implement the compiler itself has not yet been selected.

## Scope and boundaries

- Study existing compilers and official standards as implementation evidence and reference material.
- Design a new AI-native input contract; do not assume an existing programming language or compiler IR is canonical.
- Implement correct translation for explicitly supported targets and operations; reject unsupported cases.
- Preserve deterministic translation, semantic meaning, traceable conformance evidence, and defined responsibility boundaries.
- The project does **not** redesign machine language, the CPU ISA, physical memory organization, or the earlier `coding-operation-ir-compiler` / `machine-realization-core` projects.
- A limited initial target scope does **not** justify relaxing correctness requirements inside that declared scope.

## Evidence and release status

The specification, implementation, source-code presence, verified behavior, real-hardware behavior, and product release are separate states. This repository currently contains an initial documentation baseline only. Supported platforms, exact input syntax, implementation technology, and release acceptance criteria require explicit selection.

## Codex kickoff

Codex is advancing **B1: research on existing compilers and official target standards** and **B2: a concrete, explicitly unapproved compilation-contract proposal**, as defined in [the Japanese Codex instructions, section 0](docs/03_Codex施工指示書.md). Windows x64 ABI and COFF/PE sources have been added to the comparison. The ISA primary manuals and ELF gABI remain unverified, so R1 is still in progress. The initial target, AI-native input semantics, implementation language and output-production strategy remain **undecided**; implementation that commits to them requires owner adoption. No placeholder build or CI is presented as a working compiler.

Work products: [compiler and standards research (Japanese)](docs/research/01_既存技術・規格調査.md) · [unadopted initial compilation contract (Japanese)](docs/proposals/01_初期変換契約案.md)

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
