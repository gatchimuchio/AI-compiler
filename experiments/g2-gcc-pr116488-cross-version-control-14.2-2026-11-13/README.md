# GCC 14.2.0 PR 116488 prior-version control

- Record ID: `G2-HISTORY-2026-11-13-09` (B61)
- State: preregistered; the testcase outcome is `NOT_RUN` until the frozen command completes.
- Question: does the fixed GCC 15.2.0 `pr116488.c` testcase pass on the installed Debian GCC 14.2.0-19 compiler, the version named as non-reproducing in the original report?
- Procedure and scope: [preregistration](preregistration.md)

This is a cross-version control using the GCC 15.2.0 testcase and testsuite harness with a Debian-packaged GCC 14.2.0 compiler. It is not an execution of the GCC 14.2.0 release's own testsuite and does not establish the behavior of every GCC 14 build, target, or test. It does not close G2 or change an adopted AICompiler contract.
