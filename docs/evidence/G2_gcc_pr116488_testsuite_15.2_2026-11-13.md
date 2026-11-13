# GCC 15.2.0 PR 116488 testsuite run

- Record ID: `G2-HISTORY-2026-11-13-08` (B60)
- Preregistration and amendments: [fixed scope and procedure](../../experiments/g2-gcc-pr116488-testsuite-15.2-2026-11-13/preregistration.md)
- Machine-readable result: [results.json](../../experiments/g2-gcc-pr116488-testsuite-15.2-2026-11-13/observations/results.json)
- Source, build, and pre-test identities: [build-and-pretest.json](../../experiments/g2-gcc-pr116488-testsuite-15.2-2026-11-13/observations/build-and-pretest.json)
- Prior selected-test observations: [B59 GCC 15.1.0](G2_gcc_pr116488_testsuite_15.1_2026-11-13.md) and [B58 GCC 15.3.0](G2_gcc_pr116488_testsuite_2026-11-13.md)
- Repository baseline before preregistration: `054e3c65d2636a8617ce670b85dafb4605a9dd93`
- Preregistration commit: `74b95195df6aa7fe2116039e5dfd5d4e3c5424a2`
- Pre-test identity commit: `cd6b26d0163ab3aba435427c00ca0ebd67fce794`

## Result

The unmodified GCC 15.2.0 C frontend passed its own `gcc/testsuite/gcc.dg/torture/pr116488.c` regression on native Linux x86-64. The single `make check-gcc RUNTESTFLAGS='dg-torture.exp=pr116488.c'` run exited 0. DejaGnu recorded **16 expected passes**: eight torture-option variants, each with one excess-diagnostics check and one executable run. The selected `.sum` contains no `FAIL`, `XPASS`, `XFAIL`, `UNSUPPORTED`, `UNRESOLVED`, or `UNTESTED` status and contains results only for this testcase.

The variants were `-O0`, `-O1`, `-O2`, `-O3 -fomit-frame-pointer -funroll-loops -fpeel-loops -ftracer -finline-functions`, `-O3 -g`, `-Os`, `-O2 -flto -fno-use-linker-plugin -flto-partition=none`, and `-O2 -flto -fuse-linker-plugin -fno-fat-lto-objects`. The testcase applies `-fno-forward-propagate` and uses a run directive.

## Source and build identity

The tested release was `releases/gcc-15.2.0`, annotated tag object `dcd428f94ffb464418f996ffb70dfa398f5caa3f`, peeled commit `5115c7e447fc07457443df874bf57840e8316d5f`. The official GCC remote and `gcc-mirror/gcc` both returned those identities, with byte-identical 145-byte captures (SHA-256 `7e633ea387a82160c86de5c990706670aee531c1c974c81e17662a34db286c3d`). The source reports `gcc/BASE-VER` `15.2.0`.

The 101,056,276-byte archive came from GCC's official `gcc.gnu.org` release directory. Its SHA-512 matched the published `sha512.sum` entry: `89047a2e07bd9da265b507b516ed3635adb17491c7f4f67cf090f0bd5b3fc7f2ee6e4cc4008beef7ca884b6b71dffe2bb652b21f01a702e17b468cca2d10b2de`. The archive SHA-256 was `438fd996826b0c82485a29da03a72d71d6e3541a83ec702df4271f6fe025d24e`. Before and after extraction, the release testcase was 399 bytes with SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1` and Git blob `b2af84fef43040e7d9651386f52930b97a70d469`. The 40,217-byte `gcc/ext-dce.cc` SHA-256 was `f6b40d2ec541e522d98d9c9801c7427c6a1c0fc6f74dddbdc976b0036a07d76f`, Git blob `67ec92a42878ac1ef1e133cd57a163f1ad4a3159`.

GCC's bundled prerequisites passed its `--sha512` verification. Configure used host Debian GCC 14.2.0-19 and target `x86_64-linux-gnu` with `--enable-languages=c --disable-bootstrap --disable-multilib --disable-nls --disable-werror`. Both `make -j4 all-gcc` and `make -j4 all-target-libgcc` exited 0. The built `xgcc` reports GCC 15.2.0, target `x86_64-linux-gnu`; its runtime lookup resolves to this B60 build tree. The selected testsuite's native configuration was `x86_64-pc-linux-gnu`.

The testsuite used the GCC 15.2.0 source's own `gcc.exp`, test source, and local `unix` board with user-local GNU DejaGnu 1.6.3, Expect 5.45.4, Tcl 8.6.16, and gawk 5.3.0 reused from B58. DejaGnu warned that the global config file was unavailable, then loaded the release-local testsuite interfaces and completed the selected run. The archive, build, `.sum`, `.log`, and command output remain outside Git under `/tmp/aic-gcc15.2-pr116488-2026-11-13/`; hashes are in [results.json](../../experiments/g2-gcc-pr116488-testsuite-15.2-2026-11-13/observations/results.json).

The selected `.sum` was 1,711 bytes, SHA-256 `33db78c7975780d58bfdda158f40ef76ce8918a6eb9b7af314b99b596eae7fba`. The `.log` was 21,301 bytes, SHA-256 `5ad163aca5d75934ecefe1e38c4830a77847f886301c44896f384812bb5e8fa9`. The captured `make` stdout was 2,356 bytes, SHA-256 `d6e50379be6f859528e39364a98837b719bd791b7e09d803cbf4c2328a600659`; stderr was the 47-byte global-config warning, SHA-256 `72d955b06df83b7600639054bba0d74a4c0ae002b580a0e0b05fe5d4c2dceb7d`.

## Interpretation and limits

This is an observed pass for one GCC release build, one native Linux x86-64 configuration, and one release regression test with eight configured variants. B58, B59, and B60 now separately record this selected test passing on GCC 15.3.0, 15.1.0, and 15.2.0, respectively. The three observations do not establish the full GCC testsuite, other targets/configurations, GCC reliability rates, compatibility with AIC/0.2, long-term compatibility, or completion of G2.

The AICompiler Rust/C++ project tests were `NOT_RUN`; they are outside this GCC history observation. No AIC source, target, adopted contract, candidate format, QA threshold, or release state changed.
