# GCC 14.2.0 PR 116488 prior-version control — preregistration

- Record ID: `G2-HISTORY-2026-11-13-09`
- Batch: B61
- Repository baseline before preregistration: `edea42ea1813b1bfbe06cf86a2aca4454eb88908`
- Parent evidence: [GCC compiler operation history](../../docs/evidence/G2_既存compiler運用履歴_2026-11-13.md), [B60 GCC 15.2.0 execution](../../docs/evidence/G2_gcc_pr116488_testsuite_15.2_2026-11-13.md)
- Related reported observation: the PR 116488 report states the reporter could not reproduce on GCC 14.2 or earlier; that statement remains reporter evidence, not an execution by this project.
- B60 input source: GCC release tag `releases/gcc-15.2.0`, peeled commit `5115c7e447fc07457443df874bf57840e8316d5f`; verified archive and extracted tree are retained under `/tmp/aic-gcc15.2-pr116488-2026-11-13/`.
- Fixed testcase: `gcc/testsuite/gcc.dg/torture/pr116488.c`, 399 bytes, SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1`, Git blob `b2af84fef43040e7d9651386f52930b97a70d469`.
- Compiler under test: `/usr/bin/gcc-14`, GCC `14.2.0 (Debian 14.2.0-19)`, target `x86_64-linux-gnu`; binary SHA-256 `a23ecab8ff08f09ad8c80602c2c5df7f49e09c25905cb8975902e101bf72635f`. Compiler components and Debian package identities will be captured before execution.

## Question and evidence boundary

Does the GCC 15.2.0 release's fixed regression testcase compile and execute successfully under the installed Debian GCC 14.2.0-19 compiler on native Linux x86-64, across the same eight torture-option variants used by B58–B60?

This is a single, fixed negative-control observation prompted by the original PR report's statement that GCC 14.2 did not reproduce. It uses the GCC 15.2.0 testcase and GCC 15.2.0 DejaGnu testsuite support with a Debian-built GCC 14.2.0 compiler. The mismatch is intentional and will remain visible in the result. It is not GCC 14.2.0's own release testsuite, does not recreate the original reporter's full source/environment, and does not identify all GCC 14 builds. It does not establish full-suite status, other targets, reliability rates, compatibility, or G2 completion. AICompiler source and tests are outside this observation.

## Frozen input and procedure

1. Use only the already checksum-verified B60 GCC 15.2.0 extracted source tree and the fixed `pr116488.c` bytes listed above. Recheck the testcase SHA-256 and Git blob before creating the run setup. No archive retrieval, source modification, or testcase edit is planned.
2. Use `/usr/bin/gcc-14` from Debian packages `gcc-14`, `gcc-14-base`, `cpp-14`, and `libgcc-14-dev`, all version `14.2.0-19`. Record the compiler binary, `cc1`, `lto-wrapper`, `libgcc_s.so.1`, assembler/linker identities, package versions, target, kernel, and glibc before the test.
3. Use the GCC 15.2.0 release's `gcc/testsuite/gcc.dg/torture/dg-torture.exp`, `gcc/testsuite/lib/gcc-dg.exp`, `gcc/testsuite/lib/gcc.exp`, and `gcc/testsuite/lib/torture-options.exp`, together with user-local DejaGnu 1.6.3, Expect 5.45.4, Tcl 8.6.16, and gawk 5.3.0 retained from B58. Record their identities and hashes.
4. Create an isolated DejaGnu object directory and a `site.exp` derived from the B60 generated GCC 15.2.0 testsuite configuration. In the derived file, remove the `TEST_GCC_EXEC_PREFIX` assignment and redirect `tmpdir` to the B61 object directory. This ensures the GCC 14 compiler uses its own `cc1` and runtime instead of the B60 GCC 15.2.0 install tree. Do not modify the B60 source or build tree. Keep `GCC_EXEC_PREFIX` unset.
5. Run only the selected testcase once, on the `unix` board, using DejaGnu's `--tool_exec=/usr/bin/gcc-14` to set the compiler under test. The frozen invocation is:

   ```sh
   env -u GCC_EXEC_PREFIX \
     PATH=/tmp/aic-gcc15-pr116488-2026-11-13/tools/bin:/usr/bin:/bin \
     LD_LIBRARY_PATH=/tmp/aic-gcc15-pr116488-2026-11-13/tools/lib \
     /tmp/aic-gcc15-pr116488-2026-11-13/tools/bin/runtest \
     --tool gcc --tool_exec=/usr/bin/gcc-14 \
     --target_board=unix \
     --srcdir=/tmp/aic-gcc15.2-pr116488-2026-11-13/source/gcc-15.2.0/gcc/testsuite \
     --objdir=/tmp/aic-gcc14.2-pr116488-control-2026-11-13/gcc \
     dg-torture.exp=pr116488.c
   ```

   Working directory: `/tmp/aic-gcc14.2-pr116488-control-2026-11-13/gcc`.

6. Preserve the exact custom `site.exp`, command/environment, UTC start/completion, exit status, stdout/stderr, `.sum`, `.log`, per-variant statuses, and hashes under `observations/` and the external B61 root. Do not widen the selector, change compiler/options, or rerun after observing an outcome.

## Expected observation and reporting

B58, B59, and B60 each observed 16 expected `PASS` results for this testcase across eight configured variants. The registered expectation for this control is also 16 `PASS`, with no `FAIL` or `XPASS`; all other statuses and harness/build deviations will be recorded verbatim. A completed PASS supports only this one testcase under this Debian GCC 14.2.0-19 build and the mixed-version harness described above. A harness or setup failure is `NOT_RUN` for the testcase.

The external run root is `/tmp/aic-gcc14.2-pr116488-control-2026-11-13/`. No system package, global configuration, AICompiler source, adopted contract, target, QA threshold, or release state is to be changed. AICompiler project tests remain `NOT_RUN`.
