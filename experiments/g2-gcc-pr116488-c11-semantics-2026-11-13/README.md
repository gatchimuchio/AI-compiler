# GCC PR 116488 reporter sourceのC11意味境界監査 — B64

- 記録ID：`G2-HISTORY-2026-11-13-12`
- 問い：B62 reporter sourceについて、C11 draft N1570だけから期待stdoutを一意に導けるか。
- 方式：元sourceと固定N1570本文のread-only条項照合。compiler、program、AICompiler testsは実行しない。
- 事前登録：[preregistration](preregistration.md)。
- sourceはB62/B63と同一で、236 bytes、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。

B64は`plain char`の選択、範囲外signed conversion、negative signed right shiftを区別し、C11が規定する結果とimplementation-defined behaviorを分けて記録する。
