# Changelog

変更履歴は、事実・作業結果・公開判断を混同せず記録する。

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
