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

2026-11-09の実査と未確認資料は[既存技術・規格調査](research/01_既存技術・規格調査.md)に、Owner未採用の候補設計は[初期変換契約案](proposals/01_初期変換契約案.md)に分けて記録する。

**既存実装を読むことと、ソースコードを複製することは異なる。** 直接流用する場合はライセンス互換、著作権、依存・改変条件、帰属を個別に確認する。既存コンパイラの設計上の都合を、新規コンパイラの必須要件に無断で置き換えない。

## 3. 独立性

`coding-operation-ir-compiler`および`machine-realization-core`は歴史的な比較対象にとどまる。本プロジェクトでは既存の機械語やCPU/メモリ原理を再設計せず、AI専用文字列表現から**既存規格に適合する機械命令**への変換を対象とする。

## 4. 出典・証拠の扱い

当該版から得た事実、一般化した設計原則、AI-compiler固有の新設計、未実装・未検証を明確に区別する。別リポジトリのテスト成功や性能をAI-compilerの実績として記載してはならない。

## 5. 2026-11-10 追加比較――Cloud Codex単独運用規律

2026-11-10、個人/グローバル・カスタム指示がCloud Codexの各環境へ常に渡ることを前提にせず、リポジトリ局所の`AGENTS.md`を自己完結させる目的で、下記の**当該版のAGENTS原文**を参照した。変更対象はAI-compilerの統治文書のみであり、他リポジトリの機能・実績・現在status・技術設計・非公開原理を取り込まない。2026-11-02の初期参照記録（§1）は当時の史料として保持し、そのSHAを後から書き換えない。

| 参照原文 | 観測した`main` HEAD | `AGENTS.md` blob SHA | 共通運用原則の抽出／明示的非継承 |
|---|---|---|---|
| [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA/blob/main/AGENTS.md) | `d907eaaa101ad04a29f83584b1ff39995fbb7d71` | `b34836b972ce58c9606d4413744a505e336e9509` | 日本語単独規律、意味・責任・媒体交換、観測型・履歴・QAの分離／非継承：Core三機・Layer0・学習・旧配布・試験数 |
| [konjac-Translation](https://github.com/gatchimuchio/konjac-Translation/blob/main/AGENTS.md) | `4a2dcaf4a507d5e1620168b5f45c20df85bc9791` | `51a4c52e7923c0ba896edd1f0ba1cf94184a93cd` | 境界保持・未実証止まり・実装媒体の交換／非継承：翻訳/ASR/OCR/字幕・製品固有採否 |
| [BLUE-TANUKI](https://github.com/gatchimuchio/BLUE-TANUKI/blob/main/AGENTS.md) | `ed0460011a5f4105ba9ae42e52582be89e3d5da2` | `382fc0d97143f2cd3aa22182e4ab3c5e6af06432` | QE全系→BUILD→実物QC→対象別QA、境界・履歴・未接続の顕在化／非継承：HDS-BRAIN、承認段階、通信チャネル、5OS、製品進捗 |
| [D4-POCKET](https://github.com/gatchimuchio/D4-POCKET/blob/main/AGENTS.md) | `263fd01231816ff87ab7b756c29aa2ed38ffbf71` | `60419a048aa25417fb2431d432153e7648fc15c3` | 設計/施工/使用/責任の分離、実consumer、QC/QA・Git差分保全／非継承：UI、Flutter/Broker、tag運用、5OS要件・合格値 |
| [MARS](https://github.com/gatchimuchio/MARS/blob/main/AGENTS.md) | `36deba02a407f1330da384a6da8e2d1068a81a6d` | `80fb8d4cacc3aa671bd977d9d0f254a1bd37b7c6` | 原文/分析/仮説/再現を分離し参照可能に保全／非継承：歴史的鉄道予約システムの仕様・再現実験 |
| [NOTNN-LLM-MINIDORA](https://github.com/gatchimuchio/NOTNN-LLM-MINIDORA) | `9307d7e2b1799750ecef1957d85128fcbe403d02` | 2026-11-02の初期監査資料を継続参照（本追補でAGENTS原文再取得なし） | 既存§1の日本語正本・主語/目的・実証分離を維持／非継承：非ニューラルLLM固有機能/実証/秘密 |

AI-compiler側で観測した原文：`AGENTS.md`旧blob `f09966a9a710e4b9108a8593410d34bcefcd617c`、`docs/01_統合仕様書.md`、`docs/02_工程表.md`、`docs/03_Codex施工指示書.md`、`docs/proposals/01_初期変換契約案.md`、`docs/research/01_...06_...`およびRust/C++/assembler testの現行リポ構成。上表のGit HEAD/AGENTS blobは**参照した時点の同一性**であり、将来の最新状態の保証や内容の本repoへのライセンス移転ではない。本追補が採用するのは共通運用の原則だけ。AIC/0.2、target、規格未決定、R4 QA、公開可否はAI-compiler自身の三点正本とOwner採用でのみ変わる。

原文の所在は各Git URLと上記SHAで再取得可能。分析/改訂結果は本repoの`AGENTS.md`と`CHANGELOG.md`に分離して記載し、要約が原文を代替しない。
