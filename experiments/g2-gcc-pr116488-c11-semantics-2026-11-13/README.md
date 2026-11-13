# GCC PR 116488 reporter sourceのC11意味境界監査 — B64

- 記録ID：`G2-HISTORY-2026-11-13-12`
- 問い：B62 reporter sourceについて、C11 draft N1570だけから期待stdoutを一意に導けるか。
- 方式：元sourceと固定N1570本文のread-only条項照合。compiler、C testcase、AICompiler testsは実行しない。
- 事前登録：[preregistration](preregistration.md)。
- sourceはB62/B63と同一で、236 bytes、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。
- 結果：N1570だけから一意のstdoutは決まらない。plain `char`、範囲外のsigned conversion、negative signed right shiftにimplementation-defined choiceが残る。
- 条文照合：[semantics audit](../../docs/evidence/G2_gcc_pr116488_c11_semantics_2026-11-13.md)、[machine-readable trace](observations/semantics-audit.json)、[source/standard identity](observations/source-capture.json)。

B64は規格に基づくportable oracleと、実装choiceを固定したconditional traceを区別して記録する。独立oracle、compiler、consumer、GCC testsuite、AICompiler testsは**NOT_RUN**。外部raw source/PDF/textのcapture manifestは[ここ](observations/capture-manifest.json)。
