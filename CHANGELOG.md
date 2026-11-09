# Changelog

変更履歴は、事実・作業結果・公開判断を混同せず記録する。

## 2026-11-09 — R3/QC coverage follow-up

- Accepted AIC fixturesを1 byteごとに置換・削除・切断し、成功parse後のvalidatorまで含めてpanicしないことを検査。
- 48-round arithmetic/equality functionを生成し、disp8 rangeを超えるstack slotと144 instructionsを含むobjectを519 inputsでreference interpreterと照合。
- Debug/release suiteは各23 Rust unit、3 CLI、4 E2E tests。Clippy `-D warnings`、rustfmt checkも再実行して通過。
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
