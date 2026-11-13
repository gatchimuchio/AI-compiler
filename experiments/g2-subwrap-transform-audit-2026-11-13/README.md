# G2 wrap減算の変換経路監査

一つの固定`u64` wrap減算について、GCCのtree/RTL dump、rustcが出力したLLVM IR、AIC/0.2の実装sourceを既存object/disassemblyに対応づける限定監査。事前条件は`preregistration.md`、結論と限界は[`docs/evidence/G2_u64減算compiler変換経路_2026-11-13.md`](../../docs/evidence/G2_u64減算compiler変換経路_2026-11-13.md)、生dump/IRとargvは`observations/`を参照。

Diagnostic objectとdumpのraw copiesは`observations/`に保存し、object binariesはGit外の`/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/transform-audit/`に保持する。AIC/GCC/Rustの元比較記録・consumer結果は`../g2-subwrap-compiler-compare-2026-11-13/`にある。テストスイート、consumer再実行、時間測定は行っていない。
