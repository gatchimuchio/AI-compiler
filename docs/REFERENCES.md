# 参照元と継承・非継承の境界

- 観測日：2026-11-02
- 目的：初期README、ライセンス、AGENTS、品質・Git規律を既存3リポジトリの**現行版**に照らして構成する。
- 本書の参照は、別リポジトリの機能・実績・技術仕様を自動継承することを意味しない。

## 1. 参照した3リポジトリ

| リポジトリ | 確認した`main` HEAD | 採用した汎用要素 | 採用しない要素 |
|---|---|---|---|
| [MINIDORA](https://github.com/gatchimuchio/NOTNN-LLM-MINIDORA) | `9307d7e2b1799750ecef1957d85128fcbe403d02` | 日本語正本、対象・目的・手段、実証と主張の区別、境界付きGit運用 | HDS/LLM核、ベンチマーク結果、非公開理論、MINIDORA固有機能 |
| [BLUE-TANUKI](https://github.com/gatchimuchio/BLUE-TANUKI) | `76a5f5c7c019ad5164b1455761eac1da9d60adbe` | QE→実物QC→対象別QA、責任境界、証拠と公開判断の分離 | HDS-BRAIN、Runtime authority、承認モデル、チャネル実装、製品進捗 |
| [D4Pocket](https://github.com/gatchimuchio/D4-POCKET) | `0cb7d2e4204e1b32bfd108f9ecda29434c1ac3e5` | 現行正本、未接続と未検証の区別、Git保全、安全・責任境界 | GUI/Flutter/Rust Broker、5OS製品要求、過去のQC/QA合格値 |

3リポジトリの当該現行版は、ソフトウェア構成物にApache-2.0、文書成果物にCC BY 4.0を割り当てている。古いライセンス情報を優先しない。

## 2. 既存コンパイラ・規格の参照候補

- [LLVM Code Generator](https://llvm.org/docs/CodeGenerator.html)：ターゲット別命令生成と機械語出力の既存実装の理解。
- [GCC Machine Descriptions](https://gcc.gnu.org/onlinedocs/gccint/Machine-Desc.html)：CPU別命令パターンと生成制約の理解。
- [CompCert](https://compcert.org/)：入力からターゲットへ意味を保存する変換の検証手法。
- CPU/ISA、ABI、OS API、実行形式の公式規格：対応ターゲットを選定してから、必要な版を特定する。

**既存実装を読むことと、ソースコードを複製することは異なる。** 直接流用する場合はライセンス互換、著作権、依存・改変条件、帰属を個別に確認する。既存コンパイラの設計上の都合を、新規コンパイラの必須要件に無断で置き換えない。

## 3. 独立性

`coding-operation-ir-compiler`および`machine-realization-core`は歴史的な比較対象にとどまる。本プロジェクトでは既存の機械語やCPU/メモリ原理を再設計せず、AI専用文字列表現から**既存規格に適合する機械命令**への変換を対象とする。

## 4. 出典・証拠の扱い

当該版から得た事実、一般化した設計原則、AI-compiler固有の新設計、未実装・未検証を明確に区別する。別リポジトリのテスト成功や性能をAI-compilerの実績として記載してはならない。
