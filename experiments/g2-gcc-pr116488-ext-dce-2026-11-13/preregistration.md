# GCC PR 116488 RTL `ext_dce` pass 観測 — B63 preregistration

- 記録ID：`G2-HISTORY-2026-11-13-11`
- batch：B63
- 作業base：`cb25b634685f4852914821afd597aeaa016664de`
- 目的：B62と同じreporter sourceについて、既存compilerの内部変換結果を一段記録する。AICompiler製品code、採用済みcontract、target、候補、QA閾値は変更しない。
- 関連記録：[B62 reporter source reproduction](../../docs/evidence/G2_gcc_pr116488_reporter_reproduction_2026-11-13.md)、[B62 pretest identity](../g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/pretest-identity.json)

## 問いと固定範囲

GCC 15.2.0 release compilerのRTL `ext_dce` pass出力を、reporterの元`small.c`で`-O3`と`-fno-forward-propagate -O3`の2条件について取得する。比較対象はこのpassのdumpとassembly artifactだけとする。B63のcompile-only invocationは条件ごとに1回、計2回。生成programは実行しない。

入力sourceはreporterの14-line `small.c`と同一で、236 bytes、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。B62の外部source captureから各case directoryへbytesを変更せずcopyする。GCC 15.2.0 driver/`cc1`、source archive、source commit identityは[pretest identity](observations/pretest-identity.json)に記録した。

## passとoption順を選んだ根拠

GCC 15.2.0の`gcc/ext-dce.cc`はpass名を`ext_dce`とし、gateを`flag_ext_dce && optimize > 0`と記述する。`gcc/passes.def`では`pass_ud_rtl_dce`の後、`pass_combine`の前に登録されている。`gcc/common.opt`は`-fext-dce`を説明する。testcase compileの前にread-only `-Q --help=optimizers` queryを実行し、`-O3`で`-fext-dce`がenabled、reporter指定順で`-fforward-propagate`がdisabledと確認した。query command/output/status/hashはpretest記録に保存した。

dump optionはGCC 15.2.0 testsuiteのdirectiveでも使う`-fdump-rtl-ext_dce`を固定する。reporter source、optimization level、flagの順序を維持し、`-fext-dce`を追加せず、optionを並べ替えない。

## commandと実行予算

各条件は新規外部working directoryを使い、copyした`small.c`のhashをcompiler起動前に確認する。既存B60 GCC 15.2.0 buildと`-B` prefixを使い、次のcommandを一度ずつ実行する。

```text
/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -O3 -fdump-rtl-ext_dce -S small.c -o small.s
/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -fno-forward-propagate -O3 -fdump-rtl-ext_dce -S small.c -o small.s
```

各compileは`/usr/bin/timeout --signal=TERM --kill-after=1s 60s`で囲み、外側controllerにも65秒timeoutを設定する。`PATH`はB60 GCC buildと一致する。`LD_LIBRARY_PATH`は同じsearch directory順序と解決先を保つ。B63 runnerは冗長な`/./`表記を正規化する（B60 log 669 bytesに対し653 bytes）一方、先頭の空elementを保つ。`LC_ALL=C`、`LANG=C`、`GCC_EXEC_PREFIX` unset。runnerはargv、cwd、environment、UTC時刻、status、stdout/stderr bytes/hash、assembly hash、`ext_dce` dumpファイルの全件・サイズ・hashを記録する。生成programは実行しない。外部`runs/`は起動前に存在せず、再利用・上書きを拒否する。

## 判定と停止条件

- `PASS`はcompileがexit 0で、`small.s`と1つ以上のnon-empty `ext_dce` dumpを得たケースを示す。これはdiagnostic capture成功であり、dumpが意味保存や正当性を証明する意味ではない。
- identity mismatch、compile error/timeout、またはassembly/dump欠落を`FAIL`で記録する。部分出力を保持し、source/flag/compilerを変えて再実行しない。
- dump contents間の差・同一性に事前期待を置かない。観測された差と限界を記録し、正しさのoracleへ昇格させない。
- compiler/tool/source identityがpretestと一致しない、または`runs/`が既にある場合は最初のcompile前に停止する。
- native consumer、独立semantic oracle、GCC 14.2.0 compile、GCC full testsuite、AICompiler project testsはB63では**NOT_RUN**。

外部rootは`/tmp/aic-g2-gcc-pr116488-ext-dce-2026-11-13/`。preregistrationとpretest identityは最初のcompile前にcommitする。
