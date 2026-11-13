# GCC 14.2.0 PR 116488 cross-version control

- Record ID: `G2-HISTORY-2026-11-13-09` (B61)
- Result: **PASS**, one frozen execution
- Question: Does the fixed GCC 15.2.0 `pr116488.c` release testcase pass under the installed Debian GCC 14.2.0-19 compiler on native Linux x86-64?

## Frozen inputs and scope

The testcase is GCC 15.2.0 `gcc/testsuite/gcc.dg/torture/pr116488.c` (399 bytes; SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1`; Git blob `b2af84fef43040e7d9651386f52930b97a70d469`). The testcase and test harness came from the checksum-verified GCC 15.2.0 archive used in B60. The compiler under test was `/usr/bin/gcc-14`, Debian GCC `14.2.0-19`, target `x86_64-linux-gnu`. The run used GCC 15.2.0 DejaGnu testsuite support and the `unix` board. Compiler, harness, host, packages, tools, and `site.exp` identities are in [pretest-identity.json](../../experiments/g2-gcc-pr116488-cross-version-control-14.2-2026-11-13/observations/pretest-identity.json).

This was deliberately a mixed-version control. It did not use the GCC 14.2.0 release's own testsuite sources or harness, and it did not reconstruct the original reporter's entire source and environment.

## Execution and observed result

The preregistered command ran once on 2026-11-13 from `/tmp/aic-gcc14.2-pr116488-control-2026-11-13/gcc`, using `/usr/bin/gcc-14` as `--tool_exec`. It exited 0 in 1.045868749 seconds. The selected testcase produced exactly 16 `PASS` statuses and zero `FAIL`, `XPASS`, `XFAIL`, `UNSUPPORTED`, `UNRESOLVED`, or `UNTESTED` statuses. Each of the eight torture-option variants produced one diagnostics pass and one execution pass:

- `-O0`, `-O1`, `-O2`, `-O3 -g`, and `-Os`
- `-O3 -fomit-frame-pointer -funroll-loops -fpeel-loops -ftracer -finline-functions`
- `-O2 -flto -fno-use-linker-plugin -flto-partition=none`
- `-O2 -flto -fuse-linker-plugin -fno-fat-lto-objects`

The harness also ran target/effective-target support probes; the `.sum` contains results only for the selected testcase. DejaGnu printed `WARNING: Couldn't find the global config file.` The release-local GCC 15.2.0 support and `unix` board loaded, and the selected run completed.

The machine-readable outcome is in [results.json](../../experiments/g2-gcc-pr116488-cross-version-control-14.2-2026-11-13/observations/results.json) and [check-runtest-result.json](../../experiments/g2-gcc-pr116488-cross-version-control-14.2-2026-11-13/observations/check-runtest-result.json). The [capture manifest](../../experiments/g2-gcc-pr116488-cross-version-control-14.2-2026-11-13/observations/capture-manifest.json) identifies 24 retained files (23,132 bytes) in the external run directory and records their SHA-256 hashes. In particular, `gcc.sum` is 1,677 bytes with SHA-256 `1aac3c41160da710445f85e88d0031863b52fee9e1b609eb81cf95a644004591`; `gcc.log` is 13,382 bytes with SHA-256 `361803a133af16b899c85091394e35c2d37b0a192cd4c60215a6d69dba52d070`.

## Interpretation and limits

This result is consistent with the original report's statement that the issue did not reproduce on GCC 14.2, but this run only establishes that the fixed GCC 15.2.0 regression testcase passed under this one Debian GCC 14.2.0-19 build, target, and mixed-version harness. It does not confirm the outcome for the reporter's complete original source/environment, every GCC 14.2 build, other targets, or the GCC 14.2.0 release testsuite. It is not a full testsuite result, reliability estimate, compatibility claim, or G2 completion.

AICompiler repository tests were **NOT_RUN**; they were outside this observation. No AICompiler source, compiler contract, target, or QA threshold changed.
