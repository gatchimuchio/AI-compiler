# GCC 15.2.0 PR 116488 regression execution

This directory preregisters B60, a single native Linux x86-64 run of GCC 15.2.0's own `gcc.dg/torture/pr116488.c` test. The fixed scope, source identity, command, expected statuses, and stop rules are in [preregistration](preregistration.md).

At this commit, only the official release checksum list and release-tag identity have been captured. The GCC source archive has not been retrieved, extracted, or built, and the testcase has not been run. The result will be recorded only after the pre-test source, build, runtime, and harness identities are frozen.

The source archive, build tree, and raw runtime outputs will remain outside Git under `/tmp/aic-gcc15.2-pr116488-2026-11-13/`. The existing B58/B59 roots and the B58 user-local test-tool prefix remain intact.
