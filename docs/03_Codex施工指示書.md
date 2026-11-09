# AI-compiler Codex施工指示書 — 初期正本

- 版：`0.3-r3`
- 作成：2026-11-02
- 更新：2026-11-09
- 対象：AI-compilerの調査・設計・実装を依頼されたCodex
- 本書だけでコンパイラ開発開始やβ版公開の新規権限を付与しない。

## 0. 現行の施工命令（2026-11-09）

**対象：** `gatchimuchio/AI-compiler`。Ownerは2026-11-09に初期R3範囲を採用した。採用範囲はAIC/0.2の`aic_main(u64) -> u64`、Linux x86-64/System V ABI/ELF64 ET_REL、Rust 2024 + Cargoのcompiler、C++20のindependent test harness、GNU asでのobject作成、harness外部link、memory/I/O/callなし。Ownerの「どっちも採用」はRust compilerとC++ test harnessの両方を役割分担して採用する意味に記録する。

**工程現況：** 既存言語・IR・DSL・データ/仕様形式を広く参照する枝地図と、深掘り保留を採用判断から分離する方針を追加した。初期索引は全言語・全版の調査完了を意味しない。LLVM/GCC/CompCert等の既存compiler、Intel SDM ID `671200`、AMD APM Vol. 3 Rev. 3.37の命令参照、およびELF Object File Format 4.2のheader/section/symbol/relocationは既存記録の範囲で照合済み。4.3はDRAFTと表示される。AMD64 psABI baselineとstable 4.2を推奨候補として記録したが、Owner未採用のためR1継続中。R2の上記実装範囲は採用済み。Rust parser→validation→reference interpreter/assembly generation→GNU as→ELF object→C++20 harnessの初期vertical sliceが接続され、local QCを再観測した。対象別QA、形式証明、β公開は未成立・未承認。

### B1／R1の継続

既存compiler・ABI資料の限定調査は`docs/research/01_既存技術・規格調査.md`、言語・表現の開放型枝地図と深度方針は`docs/research/02_既存言語・表現参照地図.md`、構文・検証・表現境界、構造化編集/版進化、制御/効果/状態、整数/数値境界、メモリ参照/寿命/アクセス境界、診断義務/位置情報/エラー回復の限定比較は順に`docs/research/03_入力表現・構文・検証境界比較.md`〜`08_診断・位置情報・エラー回復比較.md`に記録する。LLVM/GCC/CompCertは既存compiler調査の候補群であり、言語・表現全体の代替ではない。深掘り保留は理由・現在深度・再開条件付きで残し、「未調査」「保留」「不採用」を分ける。現行出力命令をIntel SDM/AMD APMの一般命令条項と照合し、ELF Object File Format 4.2のgABI共通object規則を確認済み。baselineと参照版の推奨候補はOwner未採用である。引き続き対象familyを除外せず、AI入力の編集・意味・診断・版進化・メモリ参照等の設計問いを横断し、優先枝の一次資料調査を広げる。CPU floor/ELF版がOwner採用された時は条項をその条件で再照合する。未確認を明示し、host executionを公式適合の証明に置き換えない。

### B2／R2の採用記録

採用内容、詳細grammar/semantics、比較案、残るOwner判断事項は`docs/proposals/01_初期変換契約案.md`および[統合仕様書](01_統合仕様書.md)に記録する。AIC-J、AArch64、Windows x64、形式的証明方式、QA閾値、β条件は採用されていない。

### B3／R3一気通貫実装とQC

1. 入口からobjectまで実接続し、未対応構文・targetを明示的に拒否する。安定codeと入力位置を含む診断を維持する。
2. CFG/type/SSA/Phi意味を参照interpreterとcodegenで同じ定義に従わせ、Phi edge parallel-copy、u64 wrap境界、branch、loopをfixtureで照合する。
3. GNU asをshell command文字列経由でなく引数配列で起動し、ET_REL objectを検査してから出力先へinstallする。linkerはcompiler内から呼ばない。
4. C++20 harnessから通常ABI callし、返却値、callee-saved register、stack状態、ELF header/symbol/relocation/disassemblyをQCする。
5. debug/release build、format、lint、CLI拒否、入力境界、決定性を検査し、各結果を対象・版・条件に結び付ける。
6. ISA/gABI完全照合、別CPU/OS、形式証明、未実行検査はPASSにせず未解決として残す。R3実装と局所QCはβ公開の許可ではない。

## 1. 着手時の確認

1. Ownerの最新指示と`AGENTS.md`を確認する。
2. `docs/01_統合仕様書.md`、`docs/02_工程表.md`、本書の現行版を確認する。
3. `main`のHEAD、既存ファイル、差分、既に成立している作業を確認する。
4. 今回の対象・非対象、入力・作用・出力、対象規格、対応バージョン、責任境界を固定する。
5. 未決定事項とOwnerによる採用が必要な事項を提示する。勝手に旧リポジトリの仕様を採用しない。

## 2. 参照調査の指示

- [参照一覧](REFERENCES.md)の3リポジトリは作業規律の参照に限定する。機能、HDS内部規則、既存テスト実績、製品statusを移植しない。
- [既存言語・表現参照地図](research/02_既存言語・表現参照地図.md)を使い、プログラミング言語、IR/bytecode、query/constraint/DSL、文法、data/schema/IDL、形式仕様等の参照枝を開いておく。候補の例示は完全一覧ではなく、各枝の登録を網羅調査完了と表現しない。
- LLVM/GCC/CompCertはcompiler implementationの一部として調べる。言語・表現の全参照母集団をこの3件に置き換えない。
- 資源制約で枝を深掘り保留にする時は、保留する対象・深度、資源上の理由、再開条件を台帳に記録する。未調査・保留・不採用・採用は別の状態である。
- 既存言語・表現は、構文、意味、type/effect、control/dataflow、validation、diagnostics、incremental editing/versioning、lowering、runtime/ABI等、今回の設計問いに照らして調べる。要点を機械的に新規設計へ継承しない。
- 根拠の優先順位は、公式規格・当該版の現行コード・検証済み動作・明記された推論とする。
- 規格上の要求と、既存コンパイラ固有の言語体系・最適化・実装都合を分ける。
- 同じ機能の再実装や無駄な資料の重複を避け、抽出した知識を新規コンパイラの設計に使う。
- ソースコードの直接流用を検討する場合は、ライセンス互換、出典、改変、依存、保守責任を事前評価する。解析と複製は同じ行為ではない。

## 3. 実装時の優先事項

1. AI専用文字列の意味を先に定義し、処理の意味が変わらない変換経路を設計する。
2. 対象CPU/OS/ABI/実行形式の条件を明示する。
3. コンパイラ内部の変換規則を構築し、機械命令生成まで一気通貫で接続する。
4. 非対応入力・不明条件は明示的に拒否する。
5. 通常の実装段階では関連する技術確認だけ行い、統合後に実物QC・対象QAを実施する。

**禁止：** 旧機械語再設計との統合、入力をC/Rust/Pythonへ変換する方式の目的化、既存LLVM IR等の無検討な正本化、モックだけの成功、未実施検査のPASS、無関係な抽象化・依存追加、試験・安全境界の削除、実機や全OSの無限定な保証。

## 4. 責任・品質・証拠

コンパイラが責任を負うのは、その対象と規格条件での入力意味・変換・生成物の適合。CPUやOSが自身の仕様を履行する責任まで引き受けない。

- 何を検証したか、何を検証していないか、どの対象版・環境で観測したかを記録する。
- 変換・出力の意味保存、規格適合、決定性、拒否動作を対応範囲内で示す。
- 実機未検証でも規格照合・解析等の証拠を保持できるが、実機動作の実証と表現しない。
- 不具合の原因が未確定なら外部実行環境の責任と決めつけず、確認した境界だけを報告する。
- 局所成立を普遍的な能力・性能・互換性に拡張しない。

## 5. 変更と完了報告

- Ownerの明示指示の範囲だけで変更し、別リポジトリへの書込み、branch/PR作成、release作業を自動開始しない。
- 意味のある実装バッチの終了時に必要な試験、diff監査、Git保全を行う。
- 変更ファイル、具体的に成立した経路、未成立箇所、検証結果/NOT_RUN、対象・版・条件、commit/remote HEAD、次作業を報告する。
- β版の最終公開判断はOwnerに残す。
