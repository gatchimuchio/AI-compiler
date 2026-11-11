# Changelog

変更履歴は、事実・作業結果・公開判断を混同せず記録する。

## 2026-11-11 — R1 Apache Avro 1.12.0 schema resolution review

- Compared the versioned Avro Specification 1.12.0 on writer/reader record-field resolution, optional reader aliases, read-time defaults, union branch selection, and Parsing Canonical Form. The pinned source is in Apache Avro `release-1.12.2` commit `8fa2067f70e3012cb3fd9a8839cd97e8c7cc1772`; the package release date and specification version are recorded separately.
- Recorded that writer-only fields are ignored, missing reader fields need reader defaults, and defaults do not make fields optional during encoding. Parsing Canonical Form omits attributes such as aliases, defaults, and documentation; it does not establish migration compatibility, source-text fidelity, or unknown-byte round-tripping. The specification says its fingerprints provide no security guarantee.
- Retained the exact upstream Markdown specification source (58,744 bytes, SHA-256 `490b2d0f96bb7ca69777329df7d4adbd1334e7b27243e3b0a37f0cd204439414`) with Apache `LICENSE.txt` (29,259 bytes, SHA-256 `5e223a61e0107f44782f855c54ff8df9f53d8182dced28ba511ab92ce41919f8`) and `NOTICE.txt` (2,552 bytes, SHA-256 `1a78544696de6cf9f06e7afaf0aa8d54c799ee8b6d9a5f796408e449a8d66373`). Official HTML retrieval size/hash and source provenance are recorded in `docs/research/04_構造化編集・版進化比較.md`.
- Avro runtime, compatibility tests, and performance measurements were not run. No AIC feature or version policy was adopted; documentation-only change, compiler tests not run.

## 2026-11-11 — R1 Tree-sitter incremental parsing review

- Compared Tree-sitter v0.27.2 (released 2026-11-07; tag commit `3fb7be49145acfb07e33b61ed5689e2b6771b2e9`) with the current AIC parser on concrete syntax trees, source positions, edit mapping, old-tree reuse, and changed-range reporting.
- Recorded the requirement to apply `TSInputEdit` through `ts_tree_edit` before reparsing with the edited tree, plus the separate update requirement for retained `TSNode` values. Changed ranges describe syntactic hierarchy changes and may exceed the exact edit.
- Pinned official documentation and API source by tag, retrieval size, and SHA-256 in `docs/research/03_入力表現・構文・検証境界比較.md`. Findings do not establish source-text round-tripping, persistent IDs, semantic validation, or measured performance; no AIC feature was adopted.
- Documentation-only update; Tree-sitter runtime and benchmark observations were not run, and compiler tests were not run.

## 2026-11-11 — Zig 0.17.1 integer boundary semantics review

- Compared the adopted AIC/0.2 `u64` wrap contract with Zig's versioned official Language Reference on compile-time integer literals, runtime widths, overflow checks, wrapping/saturating operators, overflow reporting, and cast versus truncation.
- Recorded compile-time detection, safety-checked runtime behavior, and unchecked illegal behavior separately. The comparison is limited to the Zig 0.17.1 documentation; no family-wide or cross-version claim is made.
- Fixed the download-index entry and documentation body by version, retrieval date, size, and SHA-256. No Zig compiler was available for a local observation (`NOT_RUN`); no Zig code, AIC operation, or input-contract behavior was adopted.
- Documentation-only update; compiler tests were not run.

## 2026-11-10 — Cloud Codex単独運用向けAGENTS.mdの統合改訂

- AI-compilerのリポジトリ局所`AGENTS.md`を、日本語基底かつグローバル/カスタム指示に依存しない施工・監査規律へ再構成。Owner原文、主語/目的/設計者/施工者/使用者/指示者/責任者、正当性・整合性・一貫性・妥当性、観測・根因帰還・Git責任境界を明示。
- AIC/0.2の採用済み意味、pure `aic_main(u64)->u64`、u64 wrap/CFG/SSA/Phi並列意味、Linux x86-64/System V/ELF64 ET_REL、Rust compiler＋C++20 harness＋GNU asという現行分担を保持し、compiler実装言語とAI入力の意味を区別。未採用のCPU ISA feature floor/ELF規範版/QA閾値/形式証明/β公開は未決定のまま保持。
- MINIDORA-BETA、konjac-Translation、BLUE-TANUKI、D4-POCKET、MARS、既存MINIDORAの局所AGENTS・履歴から共通運用規律のみ抽出。原文参照HEADとAGENTS blob、採用／非継承の境界を`docs/REFERENCES.md`に追記し、2026-11-02の既存参照記録を保持。
- 一気通貫BUILD→実物QC→対象別QA、正本→source→object→consumerのtrace、versioned provenance、負例、未実施検査・未公開を区別する手順を統合。既存3点セット・採用/未採用状態・コード・テスト・依存・生成物を変更せず、β承認も行っていない。文書-onlyの改訂のためコンパイラ試験は`NOT_RUN`。今回の検査は文書構造・正本整合・Git差分・遠隔内容同一性に限定。

## 2026-11-09 — Memory references, lifetime, and access boundaries review

- Compared bounded primary-source questions for C11, Rust, Java SE 25, Python 3.15.2, WebAssembly Core 3.0 linear memory, and LLVM IR.
- Distinguished object lifetime, alias/access validity, bounds checks, exceptions/traps, undefined behavior, poison, and implementation-managed reclamation; no single pointer model is inferred.
- Recorded resource-based deferrals and reopening conditions for C++20, other language families and editions, managed runtimes, Wasm GC/host integration, and additional IRs.
- Kept AIC/0.2 unchanged with no memory/pointer operations; updated the AGENTS research-path index, map, roadmap, Codex instructions, and README links.
- Documentation-only change; code tests were not run.

## 2026-11-09 — Diagnostic obligations, locations, and recovery review

- Compared C11 diagnostic obligations, Java SE 25 compile-time error classification, Python 3.15.2 `SyntaxError` location fields, the current rustc JSON Output manual (with the pinned compiler build identity recorded separately), and JSON Schema 2020-12 validation output.
- Kept standards-level error detection, implementation/API diagnostic data, validator output, and parser recovery as separate claims; recorded versioned or retrieval-identified source hashes and applicability gaps.
- Recorded broad M0 branches with resource reasons and reopening conditions; no language, family, or format was rejected or removed from the open reference map.
- Compared the adopted AIC/0.2 diagnostic contract with current source at the prior HEAD; no diagnostic policy or compiler behavior was changed.
- Documentation-only change; code tests were not run.

## 2026-11-09 — Structured editing and version evolution review

- Compared the adopted AIC/0.2 input with JSON/RFC 8259, JSON Schema 2020-12, YAML 1.2.2, and Protocol Buffers on local edit identity, ordering, source fidelity, unknown elements, and schema evolution.
- Recorded specification-backed differences and separate design inferences; no AI editing benchmark or format adoption decision was made.
- Kept the broader language and representation map open, with resource-based deferrals and reopening conditions for unreviewed data/schema families.
- Updated the research map, roadmap, and Japanese/English README work-product links.

## 2026-11-09 — Control flow, effects, and state semantics review

- Compared the adopted pure AIC/0.2 subset with selected primary references for C11, Rust, Haskell 2010, ECMAScript 2025, SWI-Prolog, LLVM IR, and WebAssembly 3.0.
- Recorded differences in evaluation order, side effects, exception/search transfer, CFG versus structured control, and state rollback; source/version/implementation limits remain explicit.
- Kept all language families and unreviewed versions in the open map; resource-based deferrals include reopening conditions.
- Made no extension to the adopted input contract and ran no code tests because this batch changes documentation only.

## 2026-11-09 — Integer types and numeric boundary semantics review

- Compared AIC/0.2 integer wrap semantics with selected primary references for C11, Rust, Haskell 2010, Python 3.15.2, ECMAScript 2025, LLVM IR, and WebAssembly 3.0.
- Recorded distinctions among fixed-width bitvectors, signed overflow, arbitrary-precision integers, binary64/BigInt, LLVM poison flags, conversions, and implementation/build-specific conditions.
- Kept unreviewed language families, versions, numeric types, and wire/ABI representations in the open reference map with resource-based deferrals and reopening conditions.
- Made no input-contract or code changes; code tests were not run for this documentation-only batch.

## 2026-11-09 — Open language and representation reference map

- Added an open map of existing programming-language and representation families, separating language specifications, implementations, IRs/bytecode, data/schema/IDL formats, and DSLs.
- Recorded resource-based deep-review deferrals with reasons and reopening conditions. Deferred or unreviewed branches remain candidates; they are not treated as excluded or rejected.
- Added a bounded primary-source comparison of syntax, validation, specification status, and text/internal/binary representation boundaries across selected language families, compiler IRs, VM formats, and a schema language.
- Updated the canonical specification, research workflow, roadmap, README, and proposal context. The map is an initial index, not an exhaustive survey or adoption decision.

## 2026-11-09 — R1 source review and target-profile candidates

- Checked current x86-64 output forms and object metadata with `objdump`/`readelf`; compared the AMD64 psABI baseline and the stable ELF Object File Format 4.2 against the 4.3 DRAFT.
- Recorded unadopted recommendations for the baseline CPU profile and gABI-common ELF object-format reference. The Owner decision remains open; no conformance claim is made.

## 2026-11-09 — Independent test oracles

- Branch, loop-swap, multi-backedge loop, long arithmetic, and generated multiway Phi E2E fixtures now compare object output with both the reference interpreter and separate known-result models.
- Debug/release suiteは各25 Rust unit、3 CLI、7 E2E testsを通過。

## 2026-11-09 — ABI probe edge checks

- ABI probe now explicitly checks call-site 16-byte stack alignment, RSP restoration, RBP preservation, and RBX/R12–R15 preservation.
- Added a deliberately invalid `aic_main` fixture that clobbers RBP; the probe rejects it and recovers to its caller. Debug/release suiteは各25 Rust unit、3 CLI、7 E2E testsを通過。

## 2026-11-09 — R4 multi-backedge Phi coverage

- Added an object/interpreter and independent-model test for three loop-header Phis with one entry edge and two backedges. One latch swaps a Phi pair in parallel; the other preserves it. The fixture compares 258 terminating inputs: `0`, `1`, and even values through `512`.
- Debug/release suiteは各25 Rust unit、3 CLI、6 E2E testsを通過。

## 2026-11-09 — GNU assembler identity check

- Compiler now queries `as --version` with `LC_ALL=C` and rejects tools whose first banner line does not identify GNU assembler. Exact version and absolute path remain unpinned.
- Added a non-GNU tool rejection test that confirms an existing output stays intact. Debug/release suiteは各25 Rust unit、3 CLI、5 E2E testsで通過。
- Failed assembler invocations now retain captured stderr in the `E_TOOL` diagnostic; the failing-tool fixture checks the reported detail and output preservation.

## 2026-11-09 — R3 temporary-output isolation

- Predictable temporary filenames in a shared output directory could be replaced between reservation and use. Assembly/object staging now runs in a private `0700` temporary directory, with exclusive source creation, RAII cleanup, object inspection before rename, and the previous destination preserved on failure.
- Added a permission/cleanup check for temporary directories. Debug/release suiteは各24 Rust unit、3 CLI、5 E2E testsで通過。
- Crash durability and adversarial replacement of the final output path remain untested; this change does not claim them.

## 2026-11-09 — R3/QC coverage follow-up

- Accepted AIC fixturesを1 byteごとに置換・削除・切断し、成功parse後のvalidatorまで含めてpanicしないことを検査。
- 48-round arithmetic/equality functionを生成し、disp8 rangeを超えるstack slotと144 instructionsを含むobjectを519 inputsでreference interpreterと照合。
- 24段・各3 predecessorのPhi合流を持つCFGを生成し、全48 branch threshold、u64境界値、512 seeded pseudo-random inputsをobjectとreference interpreterで照合。
- Debug/release suiteは各23 Rust unit、3 CLI、5 E2E tests。Clippy `-D warnings`、rustfmt checkも再実行して通過。
- 有限のmutation/property testとhost実行は規格適合・形式証明を代替しない。ISA/gABI照合とbeta判断は引き続き未完了。

## 2026-11-09 — R3初期compiler slice

- Owner採用済み初期範囲に沿ってRust 2024 + Cargoのcompilerを実装。AIC/0.2 parser、stable diagnostics、CFG/type/SSA/Phi validation、u64 reference interpreter、Linux x86-64/System V assembly generation、GNU asによるELF64 ET_REL作成を接続。
- Phi edge copyはsourceをscratch slotにstage後、destinationへcommitし、loopのswap cycleを保持する。target mismatch、malformed input、unsupported operationは拒否。object検査後にoutputをinstallする。
- C++20 test harnessとassembly ABI probeを追加。22 Rust unit、3 CLI、3 E2E testsでbranch fixtureの全u16入力とu64境界、loop/Phi-cycle fixtureの0..255、ELF header/symbol/relocation/disassembly、RBX/R12-R15とRSP、byte determinismを検査。
- parser検査でASCII control byte、identifierの63-byte上限、malformed operation tokenとunsupported operationの区別を厳密化。CLIのunknown/duplicate optionと入力上書きも検査。
- Rust 1.99.1、g++ 14.2.0、GNU binutils 2.44を環境で観測。debug/release各profileで22 Rust unit、3 CLI、3 E2E testsが通過。Clippy `-D warnings`、rustfmt checkも通過。ISA一次資料とELF gABI照合、形式証明、cross-target QA、β公開は未成立。
- 統合仕様・工程表・proposal・research・施工指示書・READMEをOwner採用と現行実装状況に同期。過去のB1/B2項目はその時点では未採用だった履歴として残す。

## 2026-11-09 — B1/B2 review revision

- 添付の0.3-review施工指示に沿って、既存調査へMicrosoft x64 ABIとCOFF/PE資料（固定commit・参照位置・取得hash）を追記。ISA一次資料とSystem V ELF gABIは適正な別配布の同一原典を確認できず、R1を進行中のまま記録。
- 初期変換候補を改訂。AIC/0.1の曖昧さを記録し、入力引数、字句・文法、u64/bool、SSA dominance、Phi同時代入、duplicate edge/unreachable block拒否、loop/Phi循環のfixtureを候補として提示。
- 行指向textとJSON候補、Linux x86-64・Linux AArch64・Windows x64の比較、変換責任境界、実装・toolchain案と受入試験を追記。候補はOwner未採用で、docs/01_統合仕様書.mdへ採用済み仕様として反映していない。
- 工程表・READMEと整合するようCodex施工指示書を更新し、B1に未解決資料が残る間もOwner指示の範囲でB2候補を先行できると明記。B1完了・target採用・R3開始を意味しない。
- README・英文README・工程表を実際の調査進捗へ同期。コンパイラ実装・build・object生成・compiler testは未実施（NOT_RUN）。

## 2026-11-09 — Codex development kickoff preparation

- `main` の初期文書・ライセンス・AGENTS・三点正本と実装未着手状態を再確認。
- 初期未選定事項を維持しつつ、R1調査とR2採用候補の作成を具体的なCodex施工バッチとして定義。
- R0完了とR1着手可能を明確化。実装言語、初期ターゲット、入力意味の採用、コンパイラ本体と検証は**未実施**。

## 2026-11-02 — Repository initialization

- 公開リポジトリの初期READMEと英語説明を作成。
- Apache-2.0（ソフトウェア）／CC BY 4.0（文書）の適用範囲を設定。
- AGENTS.md、初期3点正本、参照・継承境界、基本のGit設定を整備。
- コンパイラの実装・検証・β版ローンチは**未実施**。本項目は製品リリースではない。
