# LLVM X86 tailcc修正のrelease follow-up

- 記録ID：G2-HISTORY-2026-11-13-04
- 調査・基準日：2026-11-13 UTC
- repository基準HEAD：b666119d591d09308e6d0659cc8bcd4731c84751
- 事前登録：[preregistration](../../experiments/g2-llvm-release-followup-2026-11-13/preregistration.md)
- 結果集約：[results.json](../../experiments/g2-llvm-release-followup-2026-11-13/results.json)
- 状態：**LLVM PR #235417のfixはmainへmerge済み。調査時点で最新のstable releaseは23.1.5、23.1.6は11月17日の予定で未公開。固定したrelease/23.x branchにも修正sourceと追加testを確認できなかった。後続releaseへの収録は未確認で、G2全体は未成立。**
- 適用：G2運用履歴の限定追跡。AIC/0.2仕様、compiler実装、候補、target、QA・公開状態は変更しない。

## 問いと結果

LLVM issue [#105223](https://github.com/llvm/llvm-project/issues/105223)のX86 tailcc stack-argument miscompileに対する[PR #235417](https://github.com/llvm/llvm-project/pull/235417)の修正commit 911cced04d8b4b72ae90f5ce7685a8534592ac8cについて、LLVM 23.1.5より後のstable releaseまたは release/23.x に含まれたかを、2026-11-13の公式release情報、GitHub API、Git refs、source snapshotから追跡した。

確認できた範囲では、fixは2026-10-27にmainへmerge済みだが、11月13日時点の最新公開stableは23.1.5だった。23.1.6は公式ページで11月17日予定とされ、release API、tag ref、git ls-remoteではまだ見つからなかった。release/23.x の固定commit c47d7823685a0e7048795e0cd2d58ad35e454ee9では、23.1.5 tagと同じX86 source bytesが得られ、fix前のstack-size式が残っていた。追加regression testのpathもそのbranchとtagの双方でHTTP 404だった。

この結果は取得した時点と固定commitに限る。23.1.6の実際の公開日・内容や後続版への収録を推定しない。

## release状況

- 公式LLVMサイトは23.1.5を2026-11-03公開済み、23.1.6を2026-11-17予定と掲載していた。
- GitHubのreleases/latestと llvmorg-23.1.5 release recordは同一responseで、tag llvmorg-23.1.5、公開時刻 2026-11-03T21:35:47Z を示した。
- releases/tags/llvmorg-23.1.6 と git/ref/tags/llvmorg-23.1.6 はHTTP 404。取得した git ls-remote 出力にも23.1.6 refはなかった。
- したがって23.1.5より新しいstable版の実装内容は、この調査時点では評価対象に存在しない。

## release/23.x の固定snapshot

release/23.x branch APIとupstream refから得たcommitは c47d7823685a0e7048795e0cd2d58ad35e454ee9。このcommitの llvm/lib/Target/X86/X86ISelLoweringCall.cpp は、llvmorg-23.1.5 tagから取得した同ファイルと長さ・SHA-256が一致した（130,483 bytes、4cdacbf4e5fb4865f5819f13a380dfde943db1e204cd3f17832eb6e487ab055c）。

このsnapshotでは、問題のlowering箇所は次の式を使っていた。

    NumBytes = GetAlignedArgumentStackSize(NumBytes, DAG);

fix commit側の式は GetAlignedArgumentStackSize(CCInfo.getStackSize(), DAG) である。修正前後のraw source bytesとcommitを固定して式を比較した。期待された llvm/test/CodeGen/X86/tailcc-stack-arg-align.ll は、branchと23.1.5 tagの両方でHTTP 404だった。また、release/23.x branch上のこのsource pathとtest pathについて2026-10-26以後のGitHub commit-history endpointを取得し、どちらも空配列だった。

以上から、**この固定branch snapshotでは**PR #235417の該当source変更とtest追加を確認しなかった。別の将来commit・別stable releaseの欠如や、別の実装形で同じ修正が存在しないことまでは主張しない。

## fixとhash訂正

PR #235417は2026-10-27 15:07:17 UTCにmainへmergeされた。GitHub APIで取得したcommit responseは修正箇所とtest追加を対応付ける。raw fileのSHA-256を再計算し、Git blob IDとも照合した。

| 対象 | commit / path | size | Git blob ID | raw SHA-256 |
|---|---|---:|---|---|
| 修正source | 911cced… / llvm/lib/Target/X86/X86ISelLoweringCall.cpp | 130,278 bytes | 6470869f3f19ad16c8dc67caff524434b0461d19 | 723becf4d35fc93113c23361919671a9c7f177281011318c563e625e73a823f7 |
| 追加test | 911cced… / llvm/test/CodeGen/X86/tailcc-stack-arg-align.ll | 7,272 bytes | df64b704a361f8f510a924cd5fb0e845075683f4 | 9a2bfde607e7b9aab27170cf599690838ed55dfb33ef85e533beb3804998a1a2 |

**訂正：** 既存の[G2運用履歴](G2_既存compiler運用履歴_2026-11-13.md)のLLVM fix commit行では、raw SHA-256のsource/testラベルを逆に記載していた。この追補と同時に値を正しいファイルへ付け替え、訂正履歴を同文書に明示した。commitとGit blob IDの対応は正しく、誤りはraw SHA-256のラベルだけだった。

## 固定したupstream evidence

capture manifest、HTTP status・取得時刻・response hash、upstream refs記録は[experiments/g2-llvm-release-followup-2026-11-13/observations](../../experiments/g2-llvm-release-followup-2026-11-13/observations)にある。raw body/source snapshotsはGit外の /tmp/aic-g2-subwrap-compiler-compare-2026-11-13/llvm-release-followup-2026-11-13/ に保持した。

| 一次資料 / 固定path | 観測 | 取得body / SHA-256 |
|---|---|---|
| [LLVM release schedule](https://llvm.org/) | HTTP 200。23.1.5公開、23.1.6予定の記載 | 18,134 bytes / 2c967581aa10571a1dba9a6e1314acfff704db53535cdd1ca20804702136581c |
| [GitHub releases/latest](https://api.github.com/repos/llvm/llvm-project/releases/latest) / [23.1.5 release](https://api.github.com/repos/llvm/llvm-project/releases/tags/llvmorg-23.1.5) | HTTP 200、両response同一、23.1.5公開時刻 2026-11-03T21:35:47Z | 各100,021 bytes / 763c2d0ff6f4012fcdcd38f2a6c16f77a75b70fb02a107c1cfb7dcd5ae303de3 |
| [23.1.6 release API](https://api.github.com/repos/llvm/llvm-project/releases/tags/llvmorg-23.1.6) | HTTP 404 | 133 bytes / 010563d2cfb8f1e87f700936ce59ee3a33364da46e3db41b0049227918ece74d |
| [23.1.6 tag ref API](https://api.github.com/repos/llvm/llvm-project/git/ref/tags/llvmorg-23.1.6) | HTTP 404 | 114 bytes / e2e3c64edab172132f77ce89a0e8c190db1cd30936a096344f45893bd5b75f8a |
| [upstream release/23.x ref](https://api.github.com/repos/llvm/llvm-project/branches/release/23.x) | commit c47d7823685a0e7048795e0cd2d58ad35e454ee9 | 5,391 bytes / 06a0f40e5d9e1d27a23eca26c57a6a6fd8decf49486420b28655e5b9bd87ae33 |
| upstream git ls-remote | release/23.x、23.1.5 tag/objectを列挙。23.1.6 refsなし | stdout / a7d08eb00f95adb8286a2d849e2ded5ea506639f5e34e87c04198278f827f973 |
| [PR #235417 API](https://api.github.com/repos/llvm/llvm-project/pulls/235417) / [fix commit API](https://api.github.com/repos/llvm/llvm-project/commits/911cced04d8b4b72ae90f5ce7685a8534592ac8c) | PR mergeとfile identity。PR response: 19,257 bytes / d5572afbe862665078ae53a86a5b240d7f9a985ab260eb3a9e9968818d277a13; commit response: 16,128 bytes / e4f8f0417e677d7cf5e6431b5a953d69fe9424495f79236d9d4446c86cee509f | API response hashes |
| [23.1.5 source](https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-23.1.5/llvm/lib/Target/X86/X86ISelLoweringCall.cpp) / branch commit source | bytes一致 | 130,483 bytes / 4cdacbf4e5fb4865f5819f13a380dfde943db1e204cd3f17832eb6e487ab055c |
| [23.1.5 test path](https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-23.1.5/llvm/test/CodeGen/X86/tailcc-stack-arg-align.ll) / branch commit test path | 両方HTTP 404 | 各14 bytes / d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed |
| source/test path history on release/23.x since 2026-10-26 | GitHub API responseは両方[] | 各2 bytes / 4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945 |

response hashは取得時点のHTTP body識別子であり、内容の真正性・網羅性を単独で証明しない。branch sourceとfix sourceのraw SHA-256、Git blob IDは別のidentityとして記録した。

## 適用限界と実施状況

- 対象はLLVM X86 backendの内部 tailcc 呼出規約と、上記固定release/tag/sourceに限定される。System V ABI全体、通常のC ABI、AIC/0.2の実装や品質への欠陥主張ではない。
- release scheduleは将来予定で、実際の公開や収録内容ではない。固定snapshotより後のrelease/branchに関する結論は UNKNOWN。
- reporter / maintainer / project release記録を独立再現とは扱わない。local LLVM build、LLVM test suite、repository test suiteは実行していない。production sourceは変更していない。
- 本追補はG2の限定された運用履歴証拠を増やすだけで、compiler比較の性能順位、品質率、長期運用母集団、互換性の網羅、G2成立を示さない。
