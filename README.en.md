# AI-compiler

**A new compiler project that translates AI-native, machine-operable text into machine instructions compatible with existing computing standards.**

> **Status (2026-11-02): repository initialization / design only.** No compiler implementation, runtime compatibility, performance gain, or product release has yet been demonstrated.

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

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
