# AI-compiler

**AIが直接生成・操作できる専用文字列表現を、既存の計算機規格に適合する機械命令へ変換する、新規コンパイラの研究開発プロジェクト。**

> **現在の状態（2026-11-12）：初期R2範囲とtarget profileのCPU floor/ELF参照版はOwner採用済み、R3一気通貫実装はWIRED、R4の局所QCを観測済み。** Linux x86-64はAMD64 psABI 1.0 `baseline`以上、gABI共通object規則は安定版ELF Object File Format 4.2を契約参照版とします。CLI target `x86_64-linux-gnu`はこのprofileのプロジェクト固有aliasです。広いR1調査、全target QA、β版公開は未完了・未承認です。

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

1. 既存の言語・IR・DSL・データ/仕様形式まで広く参照し、参照枝を保ったまま調査深度を資源に合わせて決めます。深掘り保留には理由と再開条件を記録し、未調査・保留を不採用と混同しません。
2. ISA、ABI、OSインターフェース、実行形式の公式仕様と照合し、規格上の要求と既存実装上の都合を混同しません。
3. 既存compiler implementationも参照しますが、LLVM/GCC/CompCertは広い参照母集団の一部です。例示した枝の全言語・全版を調査済みとはしません。
4. 入力形式は既存言語・既存IRの模倣を前提とせず、AIの操作効率と意味の一意性から設計します。
5. 対応環境・命令・操作の範囲は明示し、非対応対象を黙って成功扱いしません。
6. コンパイラが保証する事項と、CPU・OS等の外部実行環境が保証する事項を区別します。

## 採用済みの初期実装範囲

- 入力はAIC/0.2の単一関数`aic_main(u64) -> u64`。`u64` wrap add/sub、equality、CFG branch、Phi、returnを扱います。
- targetはLinux x86-64 / AMD64 psABI 1.0 `baseline` / System V AMD64 ABI / ELF64 ET_RELのみ。gABI共通object参照版はELF Object File Format 4.2です。GNU asへ`--64 -march=generic64`を渡します。linkerは試験harness側だけで使います。
- memory、pointer、I/O、syscall、global、複数function、call、exception、atomic/thread、float/vector、nondeterminismは未対応です。
- 初期backendはstack-slot方式で、性能最適化を目的にしません。非対応target・operationは拒否します。

この採用は初期実装を進める範囲を定めたもので、規格適合完了やβ公開の承認ではありません。現行命令formsとgABI共通条項の限定照合は行いましたが、契約条件の確定や全条項の適合確認は残っています。テスト結果を全CPU・全環境へ一般化しません。

## 対象外

- CPUのISA、機械語、メモリ構造・運用原理そのものの再設計
- 既存の`coding-operation-ir-compiler`、`machine-realization-core`、MINIDORA、BLUE-TANUKI、D4Pocketの開発内容の自動移植
- すべてのCPU・OS・命令・プログラムに対する初期版からの無限定な保証
- 未実装の機能、実機性能、商用品質の成立を先取りした公開主張

## 開発・検証と責任境界

技術的な成立判定は、**対象・条件を明示した範囲**について行います。コンパイラ側は入力意味の定義、変換の正当性、出力の規格適合性を担い、外部環境側は自らの仕様に従った実行を担います。テストや解析は適合性の証拠であり、局所的な結果を全環境へ一般化しません。

最初のβ版は対応範囲を限定できますが、**対応範囲内の品質要求を緩和しません**。初期範囲と未決定項目は[統合仕様書](docs/01_統合仕様書.md)で管理します。

## 現在の実装と検査

Rust compilerはAIC/0.2のparse、型/CFG/SSA/Phi検査、参照interpreter、x86-64 assembly生成からGNU as objectまでを接続しています。実行時に`as --version`のGNU assembler識別名を確認し、C++20 harnessから生成objectをlink・実行します。branch fixtureの全u16入力とu64境界値、loop/Phi-cycle fixtureの0..255、3つのloop-header Phiと2つのbackedgeを持ち一方でswapするloop、24段・各3 predecessorのPhi合流を持つ生成CFG、長いstack frameの結果を、参照interpreterと独立した期待値計算の両方に照合します。ELF header/symbol/relocation/disassembly、ABIのRSP/RBP/callee-saved GPR・DF・x87 CW・MXCSR control状態、同一環境でのbyte determinismも確認します。ABI probeは非default丸めモードで実行し、RBP・DF・x87 CW・MXCSR controlの破壊を拒否し、許容されるMXCSR status変更を受理します。probe自身のcaller状態復元もC++側で照合します。parserのfixture変異・切り詰め耐性も検査しています。objectはprivate一時directory内で生成し、header・section範囲と`sh_addr`/`sh_addralign`整列、文字列表・symbolと`aic_main`定義を限定検査してから出力先へinstallします。不整列section addressはELF 4.2違反として拒否します。SHT_GROUP sectionとSHF_GROUP member flagは、COMDAT等のlinker selectionでentry codeが除去され得る未対応shapeとして拒否します。SHT_PREINIT_ARRAY、SHT_INIT_ARRAY、SHT_FINI_ARRAYも、初期化・終了関数pointer arrayを追加し得る未対応shapeとして拒否します。extended numberingや解釈対象のcompressed tableは現在非対応です。現行backendが生成しないREL/RELA、dynamic表、非null未定義symbol、COMMONもinstall前に拒否します。process entry addressやprogram headerのoffset/countが非zeroの出力も、現producerの期待範囲外として拒否します。一般ELFの不正判定や、機械語のpure性・すべての再配置不在を証明する検査ではありません。これらはhost上の局所QCで、ISA/gABIの完全照合や形式的意味保存証明ではありません。

現実装の容量はsource 1 MiB、256 block、16,384命令、Phi incoming総数65,536、公開IR識別子各63 byte、assembler objectの観測量8 MiBです。GNU asのversion queryとassembly invocationはstdout/stderrをそれぞれ最大64 KiB captureし、direct childが実行を続けた場合はspawn後30秒で終了を要求します。超過は`E_LIMIT`、期限超過は`E_TOOL`で拒否します。parameterとterminatorは命令数へ含めません。CLIは上限＋1 byteまで読み、EOFを待たずに超過を拒否できます。これらは新たな実装制限で、AIC意味上の不正・総メモリやcompiler全体の時間保証・β資源契約とは区別します。assemblerのprocess descendantsはkill/waitせず、直接child終了後の追加出力も追跡しません。ELF検査では末尾NULの保証と必須名の固定長比較で、長い文字列の反復走査も除去しました。詳しくは[初期変換契約§2.5](docs/proposals/01_初期変換契約案.md)を参照してください。

2026-11-12のdebug/release suiteは各59 Rust unit tests、5 CLI tests、10 E2E testsがPASSです。実GNU asのCOMDAT `aic_main` group objectは修正前に受理され、現在はSHT_GROUP/SHF_GROUPをunsupportedとして拒否し、既存出力保持・staging cleanupを確認しました。実GNU asの三種類のinit/fini array sectionも未対応shapeとして拒否し、出力保持・staging cleanupを確認しました。assemblerのstdout/stderr capture上限ちょうど/+1、両stream同時出力、runtime期限超過、version queryとassembly両phaseのcompile経路でのtimeout時に既存出力保持・staging cleanupを確認します。production期限は30秒のまま、timeout testだけ50msを指定します。公開Rust emitterは現在のIRを再検証してmetadataの一致を確認します。空entry Phiと不一致metadataの拒否、self-edgeの3-cycle／自己保持／fanoutについて48変種×263入力を独立期待値へ照合しました。さらに4-block CFGの支配判定43,248件を別方式の経路探索へ照合し、二入口循環・交互latchのPhiを実objectで検査しました。いずれも有限な対象のQCです。検査手順：

~~~sh
export PATH="${HOME}/.cargo/bin:${PATH}"
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
~~~

Assembler capture unit testsには`/bin/sh`とGNU coreutilsの`head`、E2E testsにはGNU as/binutilsとg++ 14以降が必要です。CLIのstream試験とE2EにはGNU coreutilsの`timeout`が必要です。共通native runnerの30秒期限は局所試験用で、AICの停止性や製品性能の保証ではありません。Pinned toolchainは[`rust-toolchain.toml`](rust-toolchain.toml)を参照してください。objectを作成する例：

~~~sh
cargo run -- compile --target x86_64-linux-gnu examples/branch_phi.aic -o branch_phi.o
~~~

作業成果：[既存技術・規格調査](docs/research/01_既存技術・規格調査.md) · [既存言語・表現参照地図](docs/research/02_既存言語・表現参照地図.md) · [入力表現・構文・検証境界の初回比較](docs/research/03_入力表現・構文・検証境界比較.md) · [構造化編集・版進化の限定比較](docs/research/04_構造化編集・版進化比較.md) · [制御フロー・効果・状態モデルの限定比較](docs/research/05_制御フロー・効果・状態モデル比較.md) · [整数型・数値境界の限定比較](docs/research/06_整数型・数値境界意味比較.md) · [メモリ参照・寿命・アクセス境界の限定比較](docs/research/07_メモリ参照・寿命・アクセス境界比較.md) · [診断義務・位置情報・エラー回復の限定比較](docs/research/08_診断・位置情報・エラー回復比較.md) · [採用済み初期変換契約と未決事項](docs/proposals/01_初期変換契約案.md)

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
