# GCC 15.1.0 PR 116488 regression execution

This is the separately preregistered B59 run of GCC 15.1.0's own `gcc.dg/torture/pr116488.c` test on native Linux x86-64. Scope, pinned source/checksum, build/test commands, expected status, and stop rules are in [preregistration](preregistration.md).

Result: **16 expected passes** across eight torture-option variants: one excess-diagnostics check and one execution check per variant. `make check-gcc` exited 0. The exact release archive matched GCC's published SHA-512; the release tag object and peeled commit matched on the official GCC remote and mirror, and source file hashes/blobs matched the prior fixed tag capture. The machine-readable evidence is [results.json](results.json); interpretation and limits are in [B59 evidence](../../docs/evidence/G2_gcc_pr116488_testsuite_15.1_2026-11-13.md).

The source archive, build tree, full test logs, and compiler artifacts remain under `/tmp/aic-gcc15.1-pr116488-2026-11-13/`. B59 does not change AIC/0.2 or establish full GCC testsuite, cross-target correctness, AIC compatibility, or completion of G2.
