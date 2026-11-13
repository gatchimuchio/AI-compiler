# GCC PR 116488 reporter source reproduction — B62

- 記録ID：`G2-HISTORY-2026-11-13-10`
- 状態：事前登録済み。4件のcompile/runはまだ未実施。
- 対象：2024-08-26の元報告に含まれる`small.c`を、Debian GCC 14.2.0-19と既存GCC 15.2.0 buildで各2条件コンパイル・実行し、報告されたversion差と照合する。
- 手順・固定条件：[preregistration.md](preregistration.md)
- 元source captureのURL・hash・抽出法：[report-source-capture.json](observations/report-source-capture.json)

B61はGCC 15.2の固定regression testcaseをDebian GCC 14.2で実行した結果だった。B62は元reporter sourceの出力を扱う別の観測で、元報告時のGCC 15.0.0 experimental buildそのものを再構築するものではない。実験ファイル・raw outputは外部root `/tmp/aic-g2-gcc-pr116488-reporter-reproducer-2026-11-13/` に保持し、結果とhash manifestを完了後に追加する。AICompiler sourceとadopted contractは変更対象外。
