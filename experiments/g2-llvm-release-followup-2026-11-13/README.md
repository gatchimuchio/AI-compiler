# LLVM X86 tailcc release follow-up

事前登録は [preregistration.md](preregistration.md)、主な結論と適用限界は [調査記録](../../docs/evidence/G2_llvm_release_followup_2026-11-13.md)、機械可読の集約は [results.json](results.json) を参照。

2026-11-13 UTC時点でLLVM 23.1.5が最新公開版で、23.1.6は11月17日予定として掲載されていた。GitHub APIとupstream refsに23.1.6はまだなく、取得したrelease/23.xの固定commitでは対象のX86修正・回帰testを確認できなかった。これは固定時点の状態であり、23.1.6の将来内容は推定しない。

observations/には取得時刻・HTTP status・response body hashを含むcapture manifestと、upstream git ls-remoteの記録を保存した。raw HTTP bodyとsource snapshotはGit外の /tmp/aic-g2-subwrap-compiler-compare-2026-11-13/llvm-release-followup-2026-11-13/ に保持する。保存対象とSHA-256は調査記録とmanifestに列挙した。

本作業はpublic upstream記録の読取のみ。LLVM build/test、repository test、compiler source変更は行っていない。
