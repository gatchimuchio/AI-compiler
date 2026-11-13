# GCC 15.1.0 PR 116488 regression execution — preregistration

- Record ID: `G2-HISTORY-2026-11-13-07` (B59)
- Repository baseline before preregistration: `5098c124528485fc53299400dd70ff2d52cd69bd`
- Survey date: 2026-11-13 UTC
- Parent source record: [GCC 15.1.0–15.3.0 stable-source follow-up](../../docs/evidence/G2_gcc_release_followup_2026-11-13.md)
- Prior execution record: [GCC 15.3.0 PR 116488 test](../g2-gcc-pr116488-testsuite-2026-11-13/README.md)
- Fixed release tag: `releases/gcc-15.1.0`, commit `f33bb98ad1669db947ef9351939d242e4a88542b` (the official GCC Git remote and mirror IDs matched in the parent record)
- Fixed testcase: `gcc/testsuite/gcc.dg/torture/pr116488.c`, 399 bytes, SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1`, from the fixed release-tag capture

## Question and boundary

Does the unmodified GCC 15.1.0 release C compiler pass its own `gcc.dg/torture/pr116488.c` regression test on this executor's native Linux x86-64 target, using the test directives and GCC testsuite harness from that same release source?

This is one bounded G2 compiler-maintenance-history observation, focused on the first stable GCC 15 release. A pass applies only to this source, build, test, native target, and execution environment. It does not establish all-target or full-testsuite correctness, GCC reliability rates, AIC compatibility, or G2 completion. No AIC/0.2 source, target, adopted contract, candidate format, QA threshold, or release state is in scope.

## Frozen procedure

1. This preregistration and the retrieved official checksum-list snapshot are committed before retrieving the GCC source archive. Then retrieve only `https://gcc.gnu.org/pub/gcc/releases/gcc-15.1.0/gcc-15.1.0.tar.xz`. The saved official list is [`observations/gcc-15.1.0-sha512.sum`](observations/gcc-15.1.0-sha512.sum), retrieved from `https://gcc.gnu.org/pub/gcc/releases/gcc-15.1.0/sha512.sum` (HTTP 200, 600 bytes, SHA-256 `55430da81d652702c896bccf4be1813072fdaed4c62f600db2e6ac3cef6eeb73`). Its published SHA-512 for the archive is `ddd35ca6c653dffa88f7c7ef9ee4cd806e156e0f3b30f4d63e75a8363361285cd566ee73127734cde6a934611de815bee3e32e24bfd2e0ab9f7ff35c929821c1`. Record final URL, HTTP status, archive bytes, SHA-256 and SHA-512. Stop before extraction/build if retrieval fails, the official SHA-512 differs, `gcc/BASE-VER` is not `15.1.0`, the release tag cannot be tied to the fixed commit, or the testcase hash differs from the fixed value above.
2. Use the release archive's own `contrib/download_prerequisites --sha512` to retrieve only its bundled build prerequisites. Configure its C frontend for the executor's native `x86_64-linux-gnu` target with host GCC 14.2.0 and `--enable-languages=c --disable-bootstrap --disable-multilib --disable-nls --disable-werror`; then run `make -j4 all-gcc` and `make -j4 all-target-libgcc`. Preserve commands, logs, compiler identity and target-runtime artifact hashes. Stop and record `NOT_RUN` if setup fails; do not alter versions or flags after observing the testcase.
3. Reuse only the user-local DejaGnu 1.6.3, Expect 5.45.4, Tcl 8.6.16 and GNU gawk 5.3.0 binaries already built for B58 under `/tmp/aic-gcc15-pr116488-2026-11-13/tools/`. Their versions and executable/library hashes were checked before preregistration. This is reuse of test utilities only; use GCC 15.1.0's own `gcc.exp`, board configuration and test source. No system package, global configuration or repository source is changed.
4. Run only `make check-gcc RUNTESTFLAGS='dg-torture.exp=pr116488.c'` from the configured GCC 15.1.0 build directory, with the verified B58 tool prefix first in `PATH` and its Tcl library directory in `LD_LIBRARY_PATH`. Preserve the exact command/environment, `.sum`, `.log`, harness output, native runtime identity and per-variant status counts. Do not widen the selector or rerun with changed source/options after observing an outcome.
5. The prior GCC 15.3.0 run of this same 399-byte testcase yielded 16 passes across eight torture-option variants. Preregister the same expectation for 15.1.0: acceptance is one completed selected run with exactly the observed statuses recorded, 16 expected `PASS` results, and no `FAIL` or `XPASS`; any other status or count is reported verbatim as a deviation, without suppressing it. A build, configuration, runtime or harness failure is `NOT_RUN` for this testcase, not a pass.

## Environment and storage boundary

At preregistration, the executor is Linux x86-64, kernel `6.18.57`, host Debian GCC 14.2.0-19, GNU Binutils 2.44, GNU Make 4.4.1, GNU tar 1.35 and xz 5.8.1; 5 CPUs, 17 GiB RAM and about 22 GiB free disk. User-local test-tool versions were executed and checked. The previous B58 GCC tree and captures remain intact. The new source, build, tool-independent captures and raw outputs will be kept under `/tmp/aic-gcc15.1-pr116488-2026-11-13/`; no cleanup of B58 is part of this unit.

This record does not run AICompiler project tests or claim a project-code verification result; they remain `NOT_RUN`. The Rust/C++ project test suite is outside this bounded GCC history observation.
