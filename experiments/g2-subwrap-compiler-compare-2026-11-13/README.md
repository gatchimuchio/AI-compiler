# G2 u64 wrap減算compiler比較

同じ`u64` wrap減算を、AICompiler AIC/0.2 production経路、GCC C11 frontend、rustc Rust 2024 frontendで処理した一課題の探索比較。AIC入力はG3 run 02 E2/AIC replicate 0に保存されたQwen3応答。既知のrun結果を見た後の事後選択であり、標本benchmarkではない。

3つのobjectを同じrepository C++20 harnessとABI probeへ個別linkし、固定した22入力で実行した。toolchain、source/object/consumer hash、disassembly、object/function byte数、wrap減算oracleとの出力一致を記録する。実行時間測定や一般compiler品質の順位付けはしない。条件は`preregistration.md`、観測値は`results.json`を参照。

native objectとconsumer binaryはGit外の`/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/`に保持。source、hash、command output、disassembly、readelf記録、集計はこのdirectory内に保持する。

`observations/objdump_*.txt`と`observations/readelf_*.txt`はGNU Binutils stdoutのbyte-identical copyである。出力元が一部の行末に付けた空白もraw hashとの一致を保つため残しており、`git diff --check`ではその行だけ警告になる。
