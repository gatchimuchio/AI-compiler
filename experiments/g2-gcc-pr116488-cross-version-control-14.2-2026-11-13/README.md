# GCC 14.2.0 PR 116488 prior-version control

- Record ID: `G2-HISTORY-2026-11-13-09` (B61)
- State: complete; one frozen execution recorded as **PASS (16/16)**.
- Question: does the fixed GCC 15.2.0 `pr116488.c` testcase pass on the installed Debian GCC 14.2.0-19 compiler, the version named as non-reproducing in the original report?
- Procedure and scope: [preregistration](preregistration.md)
- Result and limits: [evidence report](../../docs/evidence/G2_gcc_pr116488_cross_version_control_14.2_2026-11-13.md)
- Machine-readable result: [results.json](observations/results.json)
- Raw output hashes and external files: [capture-manifest.json](observations/capture-manifest.json)
- Pretest source, compiler, harness, and environment identities: [pretest-identity.json](observations/pretest-identity.json)

This is a cross-version control using the GCC 15.2.0 testcase and testsuite harness with a Debian-packaged GCC 14.2.0 compiler. It is not an execution of the GCC 14.2.0 release's own testsuite and does not establish the behavior of every GCC 14 build, target, or test. It does not recreate the original reporter's full source/environment, close G2, or change an adopted AICompiler contract. AICompiler repository tests were NOT_RUN.
