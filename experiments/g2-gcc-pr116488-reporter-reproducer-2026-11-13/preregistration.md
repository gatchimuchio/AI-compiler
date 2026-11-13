# GCC PR 116488 reporter source reproduction — B62 preregistration

- 記録ID：`G2-HISTORY-2026-11-13-10`
- バッチ：B62
- 主語・目的：AICompilerのG2照合を進める。元のGCC誤コード報告にあるsource→compiler条件→生成binary→native consumer結果を、固定版で一組だけ追跡する。
- 施工開始base：`f4a1c959d03c39fa0e8e34b2312759249c88afa6`
- preregistration時のremote：`origin/main`が同一SHAであることを記録してからcommitする。
- 関連記録：[G2 compiler運用履歴](../../docs/evidence/G2_既存compiler運用履歴_2026-11-13.md)、[B61 cross-version control](../../docs/evidence/G2_gcc_pr116488_cross_version_control_14.2_2026-11-13.md)

## 問いと仮説

GCC PR 116488のreporterが2024-08-26に投稿した`small.c`を、Debian GCC 14.2.0-19と、B60で構築済みのGCC 15.2.0 compilerで実行したとき、`-O3`単独と`-fno-forward-propagate -O3`の出力はどうなるか。

報告の記載は、当時のGCC 15.0.0 experimental buildでは`-O3`が`0
-1
`、`-fno-forward-propagate -O3`が`0
0
`である一方、GCC 14.2以前では再現しない、というものだった。本観測の事前期待は、GCC 14.2.0-19と修正後GCC 15.2.0の双方で両条件とも`0
-1
`となること。異なる観測が出た場合は期待を変えず、差異として報告する。

## 入力sourceと出典

- 一次資料：<https://gcc.gnu.org/pipermail/gcc-bugs/2024-August/878977.html>
- archive本文は2026-11-13 10:12 UTCにHTTP 200で取得、5,038 bytes、SHA-256 `7f9c95825ed37234ae4d7d877a278609909a131470e716fcaa4706fda56d6f0d`。
- 報告者の`cat small.c` blockをHTML preformatted textから切り出したsourceは236 bytes、14行、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。外部source fileは`/tmp/aic-g2-gcc-pr116488-reporter-reproducer-2026-11-13/source/reporter-small.c`。bytesは変更しない。capture・抽出境界は[report-source-capture.json](observations/report-source-capture.json)に記録した。
- Sourceはreporterが実例として投稿した短いcode fragment。元メール自体はこのcompilerの規範・実行証明とは扱わず、報告者の観測として引用する。

## 固定compiler、target、実行数

- Debian compiler：`/usr/bin/gcc-14`、Debian GCC `14.2.0-19`、`x86_64-linux-gnu`。B61と同じinstalled compiler build。
- GCC compiler：`/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc`、B60で構築したGCC 15.2.0、`-B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/`。B60のcompiler/testsource identityとlocal artifact hashを再確認し、pretest recordに固定する。
- Native host：Linux x86-64。`as`、`ld`、libc、kernel/glibc、compiler binaries、`cc1`、`libgcc`、`timeout` identityを実行前に記録する。
- 2 compiler × 2 option cases = **4 compilation/execution cases、各1回**。caseごとに新規作業directoryと`a.out`を使い、source copy hashが固定値と一致することをcompile前に確認する。
- Command forms（CWDごとに`small.c`を置き、default output `a.out`を使用）：

  ```text
  /usr/bin/gcc-14 -O3 small.c
  /usr/bin/gcc-14 -fno-forward-propagate -O3 small.c
  /tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -O3 small.c
  /tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -fno-forward-propagate -O3 small.c
  ```

- 各compileは`/usr/bin/timeout --signal=TERM --kill-after=1s 60s`で実行し、外側Python controllerにも65秒timeoutを持たせる。compileがexit 0の場合にだけ、`/usr/bin/timeout --signal=TERM --kill-after=1s 5s ./a.out`で1回実行する。標準出力・標準エラー・両exit status・UTC時刻・argv・environmentを保存する。各caseは独立directoryで、既存outputを上書きしない。
- GCC 15.2.0 build invocationにはB60 testsuite logと同じ`PATH`を設定する。`LD_LIBRARY_PATH`はB60 `captures/gcc.log`に記録されたeffective search pathと同じ順序・解決先で指定する。runnerは冗長な`/./`表記を正規化するため、文字列長はB60 logの669 bytesに対して653 bytesとなる。空要素を含む先頭と各directoryは保持する。`GCC_EXEC_PREFIX`はunset。Debian GCC 14 caseではGCC 15 build prefixを使わない。全environmentはpretest identitiesとrun manifestに保持する。

### 実行前の記載補足

2026-11-13 10:18:30 UTCの実行前identity freeze時に、B60 `captures/gcc.log`の実値とrunnerの`LD_LIBRARY_PATH`を比較した。directoryの順序と正規化後の解決先は一致し、差は上記の冗長な`/./`表記のみだった。compile/runはこの補足時点で未開始である。

- runnerは`/tmp/aic-g2-gcc-pr116488-reporter-reproducer-2026-11-13/runs/`が既に存在する場合に停止し、再実行・上書きを拒否する。

## 判定、境界、停止条件

- Run-level expected observation：4件すべてcompile exit 0、native run exit 0、stdoutがbyte列`b"0\n-1\n"`。stderrは記録し、stdoutに混ぜない。
- Result `PASS`は固定した4件が上記どおり完了した場合のみ。違う出力、compile/runtime failure、timeoutは`FAIL`または`NOT_RUN`としてcase別に記録し、flag/source/compilerを差替えて再試行しない。
- Sourceの`char`符号・変換を含め、元sourceを保持する。異なるrelease間のlanguage default、distribution build、host target、元報告時のGCC 15.0.0 experimental build差は解釈限界として残す。
- このrunは単一reporter testcase・2 compiler build・Linux x86-64に限定され、full GCC testsuite、他target、原報告の2024年compiler buildそのもの、GCC全体の品質・信頼性・G2完了は立証しない。
- AICompiler project testsは対象外で**NOT_RUN**。AICompiler source、契約、QA閾値、候補採用、release状態を変更しない。
- Identity不一致、source hash不一致、対象build artifact欠落、またはtest runnerの安全境界不成立がcompile開始前に判明した場合、そのcaseはNOT_RUNで停止し、別compilerやflagへ切替えない。
