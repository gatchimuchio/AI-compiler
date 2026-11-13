# G2 u64 wrap減算の変換経路監査 — 事前登録

- 記録ID：`G2-TRANSFORM-2026-11-13-01`
- 基準コミット：`e34e18dc7d7b024a37da71c96a069af9d7bfa912`（cleanな`main`）
- 対象：既存記録 `G2-COMPARE-2026-11-13-02` の単一関数 `aic_main(x) = (x - 1) mod 2^64`
- 目的：固定済みAIC/C/Rust sourceから既存machine codeへ至る変換を、AICのparser/IR/interpreter/backend sourceと、GCC/Rustのcompiler dump/IRで説明できる範囲まで追う。
- 問い：AICが出すstack-slot命令列とGCC/Rustが出す5-byte命令列は、それぞれどの観測可能な中間表現またはbackend loweringで説明できるか。

## 固定入力・環境

既存G2比較のsource bytes、意味条件、22入力、期待出力、対象ABI、GCC/Rust flagsを変更しない。source SHA-256はAIC `6d6fce7ca6934c258c412e626a05d126a140455dd4cc1f24e4c0e9a1934fe363`、C `2a584d4ae06726e72ad0176c9c61a3fac24ae173cbda07d12fffeb8f840199bc`、Rust `f11d49996563363f2fecb47c5cf8c9a904a7e85679cfd4555ebf401be3b22842`。凍結内容と既存command recordは `../g2-subwrap-compiler-compare-2026-11-13/inputs/frozen_case.json` と `../g2-subwrap-compiler-compare-2026-11-13/observations/commands.json` を正とする。

- AIC compilerはG3 manifest記載のrepository commit `12234aab847992d12e8a974ed2d1dd1777aa7bfa` からビルドされた実行ファイル（既存記録のSHA-256 `13f0185a04b5a2ed89f0fa55c0d158ccfb6ca8ebc71bff57270a3b538e2f3f9e`）を参照する。`src/ir.rs`, `src/parser.rs`, `src/interpreter.rs`, `src/x86_64.rs`, `src/validate.rs`, `Cargo.toml`, `Cargo.lock` のgit blobが基準コミットと同一であることを事前確認済み。AICを再ビルドせず、既存disassemblyをコード経路と照合する。
- GCCは`/usr/bin/gcc`、14.2.0、実行ファイルSHA-256 `a23ecab8ff08f09ad8c80602c2c5df7f49e09c25905cb8975902e101bf72635f`。
- Rustは`/home/agent/.cargo/bin/rustc`、1.99.1 / LLVM 23.1.1、rustc実行ファイルSHA-256 `b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55`。
- 既存objectの基準SHA-256：AIC `f0e84af5e0053e3c4800cce2b2013525475df78cd439390cb169efc45e2f84e3`、GCC `1de6c84f1801ed6a8faf99d2a01078d29807a4792b453dce390075f9924c4429`、Rust `24d446b6223a22095261039aece4cd0abe6ef330188df36e61ec478efb4e5666`。

## 手順

1. AICの既存source実装を読む。`sub.wrap.u64`のparser→`InstructionKind::Sub`、interpreterの`wrapping_sub`、x86-64 emitterのoperand load / `subq` / stack-slot storeを追い、source blob・file hash・行番号を記録する。これを最適化passまたは証明済み等価変換とは呼ばない。
2. GCCでは既存C sourceと全既存flagsに、`-fdump-tree-original`、`-fdump-tree-optimized`、`-fdump-rtl-expand`、`-fdump-rtl-final`のみを追加しobjectを生成する。dumpはcompiler内部表現を観察するための受動的診断である。全dump/stdout/stderr/argv/exit code/hashを保存する。
3. Rustでは既存Rust sourceと全既存flagsの`--emit=obj`を`--emit=obj=<path>,llvm-ir=<path>`へ替え、同一invocationからobjectとLLVM IRを保存する。これは出力形式の指定であり、最適化level・target・crate設定を変更しない。全IR/stdout/stderr/argv/exit code/hashを保存する。
4. 診断付きGCC/Rust objectのSHA-256を既存objectと比較する。一致した場合のみIR/dumpと既存disassemblyの対応を論じる。不一致した場合は不一致とartifactをそのまま記録し、既存objectの説明に流用しない。
5. 保存先はGit外の`/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/transform-audit/`とし、報告には必要な抜粋とhash/command manifestを記録する。追加AI呼び出し、時間性能測定、追加入力、compiler/source/flagsの変更、consumer再実行は行わない。

## 停止・解釈条件

- compiler実行ファイルhash/version、source hash、基準flagsのいずれかが違えば、その経路のdiagnostic compileを停止する。
- dump/emit optionが受理されない場合は失敗とcommandを記録し、sourceや既存flagsを変更して出力を取り直さない。
- 既存objectとのhash不一致は失敗ではなく診断差分として保存するが、対応する既存命令列を説明する証拠とは扱わない。
- この監査は一関数・Linux x86-64の内部表現観測であり、速度、compiler一般の優劣、G2完了、他targetでの同じloweringを主張しない。テストスイートは実行しない。
