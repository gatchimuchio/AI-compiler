# GCC PR 116488 RTL `ext_dce` pass 観測 — B63

- 記録ID：`G2-HISTORY-2026-11-13-11`
- 問い：GCC 15.2.0 release buildは、reporterの元`small.c`を`-O3`とreporter指定順の`-fno-forward-propagate -O3`でコンパイルしたとき、`ext_dce` RTL dumpに何を出力するか。
- 範囲：各条件1回、計2回のcompile-only capture。生成assemblyと`ext_dce` dumpを読み取り、B62でnative runtime出力をすでに記録したためB63では実行しない。
- 事前確認：`-O3`のoptimizer queryでは`-fext-dce`がenabled、reporter指定順では`-fforward-propagate`がdisabled。query outputとsource/pass identitiesは[pretest identity](observations/pretest-identity.json)に記録した。
- option queryのargv・出力hash・status：[query記録](observations/option-query-verification.json)
- compile前のsetup修正2件：[setup記録](observations/setup-capture-errors.json)

固定条件と停止規則は[preregistration](preregistration.md)、一度限りのrunnerは[run_rtl_dump.py](run_rtl_dump.py)。結果とdump hashは、2件のcompile完了後に追加する。外部raw fileは`/tmp/aic-g2-gcc-pr116488-ext-dce-2026-11-13/`に保持する。

これはGCCのrelease build・1 testcase・2 option条件のcompiler-internal診断記録で、独立oracleやGCC全体の正当性証明ではない。AICompiler testsは**NOT_RUN**。
