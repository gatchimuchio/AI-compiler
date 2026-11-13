# GCC PR 116488 reporter sourceのC11意味境界監査 — B64 preregistration

- 記録ID：`G2-HISTORY-2026-11-13-12`
- batch：B64
- base：`e349a7b01e2fea6470f977b2d6c77d2046e0e20d`
- 関連実測：[B62 reporter reproduction](../../docs/evidence/G2_gcc_pr116488_reporter_reproduction_2026-11-13.md)、[B63 ext_dce dump](../../docs/evidence/G2_gcc_pr116488_ext_dce_dump_2026-11-13.md)

## 固定した問いと資料

B62/B63と同一のGCC PR 116488 reporter sourceについて、WG14 C11 Committee Draft N1570の条文から`stdout`の完全な期待値を一意に決定できるかを監査する。入力sourceは236 bytes、SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`。規範資料はN1570 (2011-04-12)、公式URL `https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf`、既存source capture SHA-256 `208969fec021e4f4ce87a1873765298c73e7bbb5209b1c5868967c7adfc8492e`に固定する。

確認対象はN1570 §§6.2.5p15（plain `char`のsigned/unsigned選択）、6.3.1.1（integer promotions）、6.3.1.3p3（signed conversionの表現範囲外結果）、6.5.7p5（negative signed right shift）、6.5.16p3（assignment expressionの値）、6.7.9p10（static storage duration objectの初期値）に限る。対象関数・mainの逐次評価を記号的に追い、N1570が固定する値とimplementation-defined choiceに依存する値を区別する。

## 固定条件と停止規則

- N1570本文は既存の公式URL captureから使い、SHA-256が上記と一致することを確認する。本文・hash不一致なら照合を停止する。
- compile、consumer execution、GCC query、testsuite、AICompiler project testsは行わない。B62の既存stdout/disassemblyは比較対象の記録として読むだけで、再生成しない。
- 実装固有のchoiceをN1570の要求へ格上げしない。unique stdoutが導けなければ、必要なchoiceと、それらを固定した場合のconditional traceを記録する。
- 判定対象はこの1 sourceのC11意味境界だけ。C11全体やGCCの正しさ、G2全体への一般化をしない。
