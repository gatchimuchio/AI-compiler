# AI-compiler

**AICompiler: a project for auditing existing languages' and compilers' machine-code transformation rules against actual compiled artifacts and real-world results, then designing and validating the most efficient AI-operable symbolic format and access/translation infrastructure for existing machine code.**

> **Status (2026-11-13): the owner adopted the initial R2 scope and the target profile's CPU floor and ELF reference edition; the R3 vertical slice is wired and local R4 QC has been observed.** The Linux x86-64 floor is AMD64 psABI 1.0 `baseline`, and the gABI-common object reference is stable ELF Object File Format 4.2. The CLI target `x86_64-linux-gnu` is a project-specific alias for that profile. The broader R1 survey, target QA, and beta publication remain incomplete; beta publication is unapproved.

[日本語正本](README.md) · [Specification (Japanese)](docs/01_統合仕様書.md) · [Roadmap (Japanese)](docs/02_工程表.md) · [Codex instructions (Japanese)](docs/03_Codex施工指示書.md) · [Agent rules](AGENTS.md)

## Goal

**Subject: AICompiler.** Its purpose is to establish an optimal approach and access/translation infrastructure for **existing** machine code using current knowledge, tooling, ISA, ABI, OS interfaces, and object formats. First compare existing compilers' transformation rules and conventions with their actual artifacts, execution results, constraints, and counterexamples. Then evaluate and design an efficient symbolic format that AI can generate, modify, and verify from natural-language requirements and translate into machine instructions and executable artifacts without losing declared semantics. **Optimizing or redesigning machine language itself is outside the project.**

The current AIC/0.2 implementation is an initial end-to-end translation baseline, **not evidence that an optimal AI-native symbolic format has been selected or validated**. The new G0–G6 project gates are distinct from the historical R0–R5 implementation and local-QC record.

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

The Rust compiler connects AIC/0.2 parsing, type/CFG/SSA/Phi validation, a reference interpreter, x86-64 assembly generation, and GNU as object creation. At runtime it checks that `as --version` identifies GNU assembler, then a C++20 harness links and executes the object. Tests compare every `u16` input and `u64` boundary cases for a branch fixture, inputs 0..255 for a loop/Phi-cycle fixture, a loop with three header Phis and two backedges that swaps a Phi pair on one latch, a generated 24-stage CFG with three-predecessor Phi merges, and a long stack frame against both the reference interpreter and independent expected-value models. They also inspect ELF headers, symbols, relocations, disassembly, ABI RSP/RBP, callee-saved GPRs, DF, x87 control word, and MXCSR controls, and byte-for-byte repeat output. The ABI probe uses non-default rounding modes and rejects RBP, DF, x87 control-word, and MXCSR-control violations while accepting caller-saved MXCSR status changes. The C++ consumer independently checks that the probe restores its caller's control state. Parser mutation/truncation cases are checked for panics. Objects are generated in a private temporary directory and checked for headers, section ranges and `sh_addr`/`sh_addralign` congruence, string/symbol references, and the defined `aic_main` function before installation. A misaligned section address is rejected as an ELF 4.2 violation. SHT_GROUP sections and SHF_GROUP member flags are rejected as unsupported because linker group selection can discard the entry function. SHT_PREINIT_ARRAY, SHT_INIT_ARRAY, and SHT_FINI_ARRAY are also rejected as unsupported initialization or termination hooks. Extended numbering and compressed interpreted tables are currently unsupported. The current backend also rejects REL/RELA sections, dynamic tables, non-null undefined symbols, and common symbols before installation because it does not emit them. A nonzero process entry address or program header offset/count is also rejected as outside the current producer's output profile. These output limits do not establish general ELF invalidity, machine-code purity, or absence of every relocation extension. These are local host QC results, not full ISA/gABI review or a formal semantic-preservation proof.

Current implementation capacities are 1 MiB of source, 256 blocks, 16,384 instructions, 65,536 Phi incoming records, 63 bytes per public-IR identifier, and 8 MiB of observed assembler object bytes. GNU as version queries and assembly invocations capture at most 64 KiB each from stdout and stderr, and request termination after 30 seconds when the direct child is still running. Excess capture is rejected with `E_LIMIT`; expiry is `E_TOOL`. Parameters and terminators do not count as instructions. The CLI reads at most the source cap plus one byte and can reject excess input before EOF. These new implementation limits narrow processing capacity; they do not make larger sources semantically invalid or establish total memory, compiler-wide time, or beta resource guarantees. Assembler descendants are not killed or waited for, and output produced after the direct child exits is not tracked. ELF inspection also uses the final-NUL invariant and a fixed-name comparison to avoid rescanning long strings for every reference. See [contract §2.5 (Japanese)](docs/proposals/01_初期変換契約案.md).

On 2026-11-13, both debug and release suites pass 70 Rust unit tests, 5 CLI tests, and 10 end-to-end tests. A valid COMDAT `aic_main` object from GNU as was accepted before the change and is now rejected with existing-output preservation and staging cleanup. GNU as fixtures for all three initialization/termination array types are also rejected before installation, preserving the destination and cleaning staging. Sections marked with SHF_TLS are likewise rejected; tests cover .tdata (PROGBITS) and .tbss (NOBITS), with destination preservation and staging cleanup. B43 limits nonempty SHF_ALLOC sections to nonwritable executable code; GNU as fixtures for .data, .rodata, and .bss are rejected before installation, while the producer’s empty .data/.bss sections remain accepted. B44 rejects GNU Binutils 2.44 SHF_EXCLUDE sections before installation after local as/readelf/ld observations; this is a producer output limit, not a claim of general gABI invalidity or all-linker behavior. B45 rejects ELF 4.2 SHF_MERGE sections before installation after GNU as/readelf showed an AXM executable section and GNU ld 2.44 merged duplicate bytes, changing aic_main code; the linked fixture terminated with SIGSEGV. Existing output preservation and staging cleanup are checked. This is not a claim of general ELF invalidity or all-linker behavior. Tests check exact and excess stdout/stderr capture, both streams at their exact cap concurrently, the assembler runtime deadline, and compile-path timeout behavior during both the version query and assembly phases, including preservation of existing output and staging cleanup. The production deadline remains 30 seconds; the timeout test supplies 50 ms. The public Rust emitter revalidates the current IR and checks the supplied metadata. Tests reject empty entry Phis and mismatched metadata, and compare 48 self-edge Phi variants with three-cycle, self-copy, and fanout behavior against independent gold on 263 inputs each. Another 43,248 ordinary/Phi-edge dominance decisions on four-block CFGs match an independent path-search oracle. Native fixtures also cover a two-entry cycle and alternating latch maps. These are finite QC populations. B46 rejects ELF SHF_LINK_ORDER sections before installation. A normal GNU ld 2.44 link of a real GNU as 2.44 object whose AXL .text.aic_main section linked to an ordinary .text.anchor section exited with status 42 without section garbage collection; GNU ld 2.44 with --gc-sections discarded the link-order section and failed the external aic_main reference. This is scoped to the observed linker and option, not general ELF invalidity or an all-linker guarantee. B47 (runtime clock snapshot: 2026-11-12 UTC) rejects a `.note.GNU-stack` section marked executable before installation. In GNU as/ld 2.44 observations, that marker produced `GNU_STACK RWE`; the producer-style marker without `X` produced `RW`. The two focused regressions and debug/release suites (72 Rust unit, 5 CLI, and 10 E2E tests each) pass, including destination preservation, staging cleanup, and acceptance of the normal marker. This is a GNU-specific producer-profile limit, not general ELF invalidity or a guarantee of final executable stack permissions. See [research §3.27](docs/research/01_既存技術・規格調査.md).

B48 (runtime clock snapshot: 2026-11-12 UTC) inspects GNU `NT_GNU_PROPERTY_TYPE_0` and rejects an object before installation when `GNU_PROPERTY_X86_ISA_1_NEEDED` raises the required x86 ISA level above the adopted AMD64 psABI baseline. A GNU Binutils 2.44 observation confirms that a v4 note is propagated into the linked image's `PT_GNU_PROPERTY`. Baseline notes, ISA-used notes, other property types, and no note remain accepted; the compiler does not require an ISA note. Three focused regressions and the debug/release suites (75 Rust unit, 5 CLI, and 10 E2E tests each), format/check/Clippy pass, with destination preservation and staging cleanup checked. This is a local output-profile boundary for one AIC compiler object, not a guarantee about all link inputs, the final image, or a CPU. See [research §3.28](docs/research/01_既存技術・規格調査.md).

Local checks:

~~~sh
export PATH="${HOME}/.cargo/bin:${PATH}"
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
~~~

Assembler capture unit tests require `/bin/sh` and GNU coreutils `head`. End-to-end tests require GNU as/binutils and g++ 14 or later. CLI stream and end-to-end tests also require GNU coreutils `timeout`. The common native runner has a 30-second local test deadline; this does not guarantee AIC termination or product performance. The Rust toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). To compile an object:

~~~sh
cargo run -- compile --target x86_64-linux-gnu examples/branch_phi.aic -o branch_phi.o
~~~

Work products: [compiler and standards research (Japanese)](docs/research/01_既存技術・規格調査.md) · [open language and representation map (Japanese)](docs/research/02_既存言語・表現参照地図.md) · [first syntax and validation comparison (Japanese)](docs/research/03_入力表現・構文・検証境界比較.md) · [structured editing and schema evolution comparison (Japanese)](docs/research/04_構造化編集・版進化比較.md) · [control flow, effects, and state model comparison (Japanese)](docs/research/05_制御フロー・効果・状態モデル比較.md) · [integer types and numeric boundary semantics comparison (Japanese)](docs/research/06_整数型・数値境界意味比較.md) · [memory references, lifetime, and access boundaries comparison (Japanese)](docs/research/07_メモリ参照・寿命・アクセス境界比較.md) · [diagnostic obligations, locations, and recovery comparison (Japanese)](docs/research/08_診断・位置情報・エラー回復比較.md) · [adopted initial compilation contract and open questions (Japanese)](docs/proposals/01_初期変換契約案.md)

## License

Software/source code and implementation assets: **Apache License 2.0**. Documentation, specifications, designs and explanatory material: **CC BY 4.0**. These licenses apply by artifact class; they are not an optional dual license for every file. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

[GitHub](https://github.com/gatchimuchio)
