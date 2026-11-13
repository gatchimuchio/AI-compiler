# GCC 15.2.0 PR 116488 regression execution

This directory preregisters B60, a single native Linux x86-64 run of GCC 15.2.0's own `gcc.dg/torture/pr116488.c` test. The fixed scope, source identity, command, expected statuses, and stop rules are in [preregistration](preregistration.md).

The preregistration and official checksum/tag identity captures were pushed before the source archive was retrieved. The archive and source identities match the registered values; the C frontend and target `libgcc` builds exited 0. The source, build, raw logs, and tool outputs remain outside Git. The selected testcase had not been run at the pre-test freeze; its exact command and acceptance rule remain fixed in the preregistration.

The source archive, build tree, and raw runtime outputs will remain outside Git under `/tmp/aic-gcc15.2-pr116488-2026-11-13/`. The existing B58/B59 roots and the B58 user-local test-tool prefix remain intact.
