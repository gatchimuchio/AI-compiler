# AI-compiler

**AIが直接生成・操作できる専用文字列表現を、既存の計算機規格に適合する機械命令へ変換する、新規コンパイラの研究開発プロジェクト。**

> **現在の状態（2026-11-09）：初期R2範囲はOwner採用済み、R3一気通貫実装はWIRED、R4の局所QCを観測済み。** R1はIntel/AMD ISA一次資料とELF gABIの照合が未完了で、対象別QA・β版公開は未完了・未承認です。

[English](README.en.md) · [統合仕様書](docs/01_統合仕様書.md) · [工程表](docs/02_工程表.md) · [Codex施工指示書](docs/03_Codex施工指示書.md) · [参照と継承範囲](docs/REFERENCES.md) · [開発規則](AGENTS.md)

## 主語と目的

本プロジェクトの主語は**新規コンパイラ**です。AIが計算内容を操作可能な形式で記述し、その意味を保存したまま、指定した実行環境の命令・呼出規約・実行形式に適合する生成物へ変換することを目指します。

- **入力：** AIの生成・変更・検証に適した専用の文字列表現。人間の可読性は必須要件ではありません。
- **変換：** 操作の意味を保持し、規格・制約を処理する新規コンパイラ。
- **出力：** 対象CPU・OS・ABIなどの明示した条件に適合する機械命令および必要な実行形式。
- **品質：** 対応を宣言した範囲での意味保存、規格適合性、決定性、証拠と責任境界を成立条件にします。

従来の高級プログラミング言語を、**コンパイル対象の必須の中間表現にはしません**。Owner採用により、compiler本体はRust 2024 + Cargo、独立試験harnessはC++20を使います。

## 構成原則

`AIの操作文字列 → 意味定義・検査 → 新規コンパイラ → 対象規格に適合する機械命令・実行形式`

1. 既存コンパイラを解析し、成立済みの命令生成・制約処理・検証資産を参照します。
2. ISA、ABI、OSインターフェース、実行形式の公式仕様と照合し、規格上の要求と既存実装上の都合を混同しません。
3. 入力形式は既存言語・既存IRの模倣を前提とせず、AIの操作効率と意味の一意性から設計します。
4. 対応環境・命令・操作の範囲は明示し、非対応対象を黙って成功扱いしません。
5. コンパイラが保証する事項と、CPU・OS等の外部実行環境が保証する事項を区別します。

## 採用済みの初期実装範囲

- 入力はAIC/0.2の単一関数`aic_main(u64) -> u64`。`u64` wrap add/sub、equality、CFG branch、Phi、returnを扱います。
- targetはLinux x86-64 / System V AMD64 ABI / ELF64 ET_RELのみ。compilerはGNU asでrelocatable objectを作り、linkerは試験harness側だけで使います。
- memory、pointer、I/O、syscall、global、複数function、call、exception、atomic/thread、float/vector、nondeterminismは未対応です。
- 初期backendはstack-slot方式で、性能最適化を目的にしません。非対応target・operationは拒否します。

この採用は初期実装を進める範囲を定めたもので、規格適合完了やβ公開の承認ではありません。ISA命令の一次資料とELF gABIは未確認で、テスト結果を全CPU・全環境へ一般化しません。

## 対象外

- CPUのISA、機械語、メモリ構造・運用原理そのものの再設計
- 既存の`coding-operation-ir-compiler`、`machine-realization-core`、MINIDORA、BLUE-TANUKI、D4Pocketの開発内容の自動移植
- すべてのCPU・OS・命令・プログラムに対する初期版からの無限定な保証
- 未実装の機能、実機性能、商用品質の成立を先取りした公開主張

## 開発・検証と責任境界

技術的な成立判定は、**対象・条件を明示した範囲**について行います。コンパイラ側は入力意味の定義、変換の正当性、出力の規格適合性を担い、外部環境側は自らの仕様に従った実行を担います。テストや解析は適合性の証拠であり、局所的な結果を全環境へ一般化しません。

最初のβ版は対応範囲を限定できますが、**対応範囲内の品質要求を緩和しません**。初期範囲と未決定項目は[統合仕様書](docs/01_統合仕様書.md)で管理します。

## 現在の実装と検査

Rust compilerはAIC/0.2のparse、型/CFG/SSA/Phi検査、参照interpreter、x86-64 assembly生成からGNU as objectまでを接続しています。実行時に`as --version`のGNU assembler識別名を確認し、C++20 harnessから生成objectをlink・実行します。branch fixtureの全u16入力とu64境界値、loop/Phi-cycle fixtureの0..255、2つのbackedgeを持つloop-header Phi、24段・各3 predecessorのPhi合流を持つ生成CFG、長いstack frame、ELF header/symbol/relocation/disassembly、ABI register/stack状態、同一環境でのbyte determinismを確認します。parserのfixture変異・切り詰め耐性も検査しています。objectはprivate一時directory内で生成・検査し、その後に出力先へinstallします。これらはhost上の局所QCで、ISA/gABIの完全照合や形式的意味保存証明ではありません。

2026-11-09時点のdebug suiteは25 Rust unit tests、3 CLI tests、6 E2E testsです。検査手順：

~~~sh
export PATH="${HOME}/.cargo/bin:${PATH}"
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
~~~

E2E testsにはGNU as/binutils、g++ 14以降が必要です。Pinned toolchainは[`rust-toolchain.toml`](rust-toolchain.toml)を参照してください。objectを作成する例：

~~~sh
cargo run -- compile --target x86_64-linux-gnu examples/branch_phi.aic -o branch_phi.o
~~~

作業成果：[既存技術・規格調査](docs/research/01_既存技術・規格調査.md) · [採用済み初期変換契約と未決事項](docs/proposals/01_初期変換契約案.md)

## リポジトリの正本

- [`docs/01_統合仕様書.md`](docs/01_統合仕様書.md)：目的、責任境界、成立条件、技術仕様
- [`docs/02_工程表.md`](docs/02_工程表.md)：段階、依存関係、完了条件
- [`docs/03_Codex施工指示書.md`](docs/03_Codex施工指示書.md)：AI実装担当の作業手順
- [`AGENTS.md`](AGENTS.md)：全作業の基底規律
- [`docs/REFERENCES.md`](docs/REFERENCES.md)：参照リポジトリと継承・非継承の境界

## ライセンス

成果物の種類ごとに適用範囲を分けます。ソースコード・実装資産は **Apache-2.0**、README・仕様・設計・技術文書等は **CC BY 4.0** です。任意選択型のデュアルライセンスではありません。詳細は[LICENSE](LICENSE)と[NOTICE](NOTICE)を参照してください。

## Author

**がっちむち♂ / gatchimuchio** — Concept originator / project owner

GitHub: [gatchimuchio](https://github.com/gatchimuchio)
