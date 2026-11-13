# GCC PR 116488 reporter source `ext_dce` RTL dump — B63

- 記録ID：`G2-HISTORY-2026-11-13-11`
- preregistration commit：`16a4e6632865d3b4f51b63ba8219943a92a502f8`
- 結果：2条件とも**PASS（compileと診断artifactの取得）**。GCCの正当性やsourceの意味を判定したPASSではない。
- AICompiler製品code、採用済みcontract、target、候補、QA閾値は変更していない。

## 固定した問いと入力

B62で用いたGCC PR 116488 reporterの元`small.c`について、GCC 15.2.0 release buildがRTL `ext_dce` pass後に出力する表現を比較した。sourceは236 bytes、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。`-O3`と、reporterが記載した順序を保つ`-fno-forward-propagate -O3`を各1回実行した。compiler起動前にpreregistration、source/pass/compiler identities、option query、runner、output directory absenceを確認し、pre-run stateを[attestation](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/pre-run-attestation.json)に記録した。

GCC driverはB60 official `gcc-15.2.0` archive由来のx86-64 Linux build（archive SHA-256 `438fd996826b0c82485a29da03a72d71d6e3541a83ec702df4271f6fe025d24e`、source commit `5115c7e447fc07457443df874bf57840e8316d5f`）。`xgcc` SHA-256は`d11bd49769c79c84969134620c9c7607fb8cf53d0be074e8e012b9cc47c4b297`、`cc1`は`987e7a1b2eb0b3103fa427bc5d7d4b05f9beeae4c2176d794216bbbc28648118`。pass sourceは`gcc/ext-dce.cc`、pass登録は`gcc/passes.def`、optionは`gcc/common.opt`にあり、queryで`-O3`時の`-fext-dce`が両条件ともenabled、reporter指定順では`-fforward-propagate`がdisabledと確認した。該当source file hashesとquery bytes/hashは[pretest identity](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/pretest-identity.json)および[query verification](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/option-query-verification.json)に保持する。

## 実行結果

各条件で`-fdump-rtl-ext_dce -S`を1回だけ実行した。2つのcompileはいずれもexit 0、timeoutなし、stdout/stderrは空で、assemblyとnon-empty RTL dumpを生成した。

| 条件 | `small.s` bytes / SHA-256 | `ext_dce` dump bytes / SHA-256 | `f` functionのRTL summary insn数 |
|---|---:|---:|---:|
| GCC 15.2.0 `-O3` | 1,599 / `5c53de8d8b27e26d7ac9ed8ad746fbf49cec3ca279a1faa528ee7a2f1d6f7f97` | 13,075 / `38c44965d93074e7de3cc97e264070fdb7a484f288d3921f6e8b08c9df19ad14` | 25 |
| GCC 15.2.0 `-fno-forward-propagate -O3` | 1,599 / `5c53de8d8b27e26d7ac9ed8ad746fbf49cec3ca279a1faa528ee7a2f1d6f7f97` | 13,441 / `0119636e2ad8c263351aa7c579a829447c22f64131704121bb0e993a81133323` | 27 |

`f`のRTL dumpでは、reporter指定順にQI subregを一時QI registerへ移す明示的なinsnが、byte multiply前と後続のsign extension用にそれぞれ現れた。`ext_dce` snapshot全体は同一ではないが、最終assemblyはbytes/hashが一致した。pass時点の中間表現差が後段で同じassemblyにまとまった観測であり、この差を最終命令差、実行時差、または挙動差と解釈しない。process固有のpointer IDやbasic-block番号を含むraw dumpは外部rootにそのまま保持し、比較範囲・hash・insn数は[post-run review](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/postrun-machine-review.json)に記録した。

## 解釈と限界

- `PASS`は固定した2回のcompileが成功し、assemblyと`ext_dce` dumpが保存できたことを表す。RTLの意味保存やcompiler正当性を確認する独立判定ではない。
- この実験は1 source、GCC 15.2.0の1 build、x86-64 Linux、2 option条件の限定観測である。差の発生を単一passへ因果帰属するものではなく、他のversion、target、testcaseや全GCC testsuiteへ一般化しない。
- B62で既に記録したconsumer出力はB63では再実行していない。native consumer、独立semantic oracle、GCC testsuiteは**NOT_RUN**。AICompiler project testsも**NOT_RUN**。
- B62のGCC 15.2.0 runtime observationでは同じ2 option条件のbinaryとstdoutが同一だった。B63のdump差はこの既存観測を補う内部artifactであり、GCC 15.0 experimental reporter差を再現した証拠ではない。
- B63はG2の独立oracle、広い互換性、蓄積された運用母集団を満たさず、G2全体は未成立のまま。

全compile argv、環境、UTC時刻、status、stdout/stderr、assembly/dump hashは[results](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/results.json)に記録した。raw source、2 assembly、2 RTL dumps、option-query captures、run logsを含む外部rootは`/tmp/aic-g2-gcc-pr116488-ext-dce-2026-11-13/`、27 regular-file paths・99,189 bytes。全file hashは[repository capture manifest](../../experiments/g2-gcc-pr116488-ext-dce-2026-11-13/observations/capture-manifest.json)にある。
