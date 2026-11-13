# GCC 15.1.0 PR 116488 testsuite run

- Record ID: `G2-HISTORY-2026-11-13-07` (B59)
- Preregistration and amendments: [fixed scope and procedure](../../experiments/g2-gcc-pr116488-testsuite-15.1-2026-11-13/preregistration.md)
- Machine-readable result: [results.json](../../experiments/g2-gcc-pr116488-testsuite-15.1-2026-11-13/results.json)
- Prior selected-test comparison: [B58 GCC 15.3.0 run](G2_gcc_pr116488_testsuite_2026-11-13.md)
- Repository baseline before preregistration: `5098c124528485fc53299400dd70ff2d52cd69bd`

## Result

The unmodified GCC 15.1.0 C frontend passed its own `gcc/testsuite/gcc.dg/torture/pr116488.c` regression on native Linux x86-64. DejaGnu reported **16 expected passes**: eight torture-option variants, each with one excess-diagnostics check and one executable run. The selected `.sum` contains no `FAIL`, `XPASS`, `XFAIL`, `UNSUPPORTED`, `UNRESOLVED`, or `UNTESTED` status. `make check-gcc` exited 0.

The eight variants were `-O0`, `-O1`, `-O2`, `-O3 -fomit-frame-pointer -funroll-loops -fpeel-loops -ftracer -finline-functions`, `-O3 -g`, `-Os`, `-O2 -flto -fno-use-linker-plugin -flto-partition=none`, and `-O2 -flto -fuse-linker-plugin -fno-fat-lto-objects`. The testcase applies `-fno-forward-propagate` in each variant and checks its `dg-do run` result.

## Source and runtime identity

The tested release ref is `releases/gcc-15.1.0`. Its annotated tag-object ID is `f33bb98ad1669db947ef9351939d242e4a88542b`; the peeled commit ID is `1b306039ac49f8ad91ca71d3de3150a3c9fa792a`. B59 queried both IDs from GCC's official Git remote and `gcc-mirror/gcc`; the outputs matched byte-for-byte. This distinguishes the tag object from the commit it points to. B57/B58's earlier records used “commit” for the tag-object IDs; those labels were corrected before the B59 test.

The 98,268,344-byte archive came from GCC's official `gcc.gnu.org` release directory and matched its published SHA-512 `ddd35ca6c653dffa88f7c7ef9ee4cd806e156e0f3b30f4d63e75a8363361285cd566ee73127734cde6a934611de815bee3e32e24bfd2e0ab9f7ff35c929821c1`. The source reports `gcc/BASE-VER` `15.1.0`. The 399-byte testcase SHA-256 is `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1` with Git blob `b2af84fef43040e7d9651386f52930b97a70d469`, matching the fixed tag capture. `gcc/ext-dce.cc` likewise matches its fixed SHA-256 and blob ID. There was no archive-host deviation.

The C frontend was configured for `x86_64-linux-gnu` with host Debian GCC 14.2.0, `--enable-languages=c --disable-bootstrap --disable-multilib --disable-nls --disable-werror`. `make -j4 all-gcc` and `make -j4 all-target-libgcc` both exited 0. The target runtime was built before this run because `check-gcc` does not itself build `all-target-libgcc`, while the selected testcase is a run test. The built `xgcc` reports GCC 15.1.0 and resolves `libgcc_s.so.1` to this B59 build tree.

The testsuite used GCC 15.1.0's own `gcc.exp` and test source with user-local GNU DejaGnu 1.6.3, Expect 5.45.4, Tcl 8.6.16, and gawk 5.3.0 reused from B58. The `.sum`, `.log`, build output, archive, and toolchain outputs remain outside Git at `/tmp/aic-gcc15.1-pr116488-2026-11-13/`; hashes and exact commands are in [results.json](../../experiments/g2-gcc-pr116488-testsuite-15.1-2026-11-13/results.json). No system packages or global configuration changed.

DejaGnu printed `WARNING: Couldn't find the global config file.` It loaded GCC 15.1.0's local `gcc.exp`, the `unix` board and the release testsuite interface, then completed the selected run with the recorded summary. The warning and full harness output are retained in the external capture root.

## Interpretation and limits

This is an observed pass for one GCC release build, one native Linux x86-64 configuration, and one release regression test with eight configured variants. It does not establish the GCC 15.2.0 result, the full GCC testsuite result, behavior on other targets/configurations, compiler reliability rates, compatibility with AIC/0.2, or completion of G2. B58 separately records the same selected test passing on GCC 15.3.0; the two observations do not establish the GCC 15 series as a whole.

AICompiler repository tests remain `NOT_RUN`; no AIC source, target, adopted contract, candidate format, QA threshold or release state changed. G2 remains incomplete.
