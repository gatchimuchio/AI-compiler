# LLVM X86 tailcc修正のrelease follow-up — 事前登録

- 記録ID：`G2-HISTORY-2026-11-13-04`
- 基準HEAD：`b666119d591d09308e6d0659cc8bcd4731c84751`
- 基準日：2026-11-13 UTC
- 対象：G2履歴に記録済みのLLVM issue #105223 / PR #235417 / fix commit `911cced04d8b4b72ae90f5ce7685a8534592ac8c`
- 問い：LLVM 23.1.5で未包含と記録したX86 `tailcc` stack-argument修正とregression testが、2026-11-13時点で後続安定版または`release/23.x` branchに含まれたか。
- 適用：AICompilerのG2運用実績記録の追補のみ。AIC/0.2の仕様、compiler source、候補採用、target、QA状態は変更しない。

## 探索と確定証拠の区別

この事前登録前にread-onlyの検索結果で公式LLVM scheduleが23.1.6を2026-11-17予定と示すことを見た。この発見は探索用であり、確定証拠・hash対象の代用にしない。以下の固定URL/API/refsの再取得snapshotを記録する。

## 固定した取得先と確認項目

1. `https://llvm.org/` のUpcoming Releasesから23.1.xの最新releaseと次回予定日を保存する。
2. GitHub REST API `repos/llvm/llvm-project/releases/latest`で現時点のlatest release tag/published timeを保存する。
3. `releases/tags/llvmorg-23.1.6`および`git/ref/tags/llvmorg-23.1.6`のHTTP status/bodyを取得し、release/tagの有無を区別する。
4. upstream Git refs `refs/heads/release/23.x`, `refs/tags/llvmorg-23.1.5`, `refs/tags/llvmorg-23.1.6`を照会する。
5. fix commitのX86 source patchとtest追加、23.1.5 tag source/test、当日の`release/23.x` source/testを取得し、同じ計算経路変更とPR #105223 testcaseの有無を比較する。必要なら`release/23.x` source pathのcommit historyをGitHub APIで取得する。
6. 応答status、取得UTC時刻、raw body hash、commit/tag SHA、確認したcode/test identityを記録する。生snapshotはGit外の`/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/llvm-release-followup-2026-11-13/`に置き、reportには一次source URLとhashを記す。

## 解釈・停止条件

- 公開projectのreporter/maintainer/release記録として扱い、本環境でLLVMをbuild/runせず、LLVM test suiteを再実行しない。
- `23.1.6`が未公開なら、未公開と次回scheduleだけを記録し、将来のrelease内容・fix包含を推定しない。
- branchにcode/testが見つからなくても、その時点・固定branch snapshotでの不在として記録し、全release branchや将来releaseの欠如へ一般化しない。
- API rate limit、HTTP error、git ref不一致、source取得不能はそのまま記録して該当主張を未確認にする。認証、別mirror、非公式summaryで穴埋めしない。
- 既存issue/PR/tagsへの書込、外部連絡、release操作、compiler build/test、AIC実装変更は行わない。
