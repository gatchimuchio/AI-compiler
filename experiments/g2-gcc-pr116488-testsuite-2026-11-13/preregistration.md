# GCC 15.3.0 PR 116488 regression execution — preregistration

- Record ID: `G2-HISTORY-2026-11-13-06` (B58)
- Repository baseline: `282c0cc4421d7ac738887722ccb40fe3724c2f8a`
- Survey date: 2026-11-13 UTC
- Parent source record: [GCC 15.1.0–15.3.0 stable-source follow-up](../../docs/evidence/G2_gcc_release_followup_2026-11-13.md)
- Fixed target: GCC 15.3.0 release tag commit `6d02d02c0b733e4934b8e1e497d90612116de9af`; test `gcc/testsuite/gcc.dg/torture/pr116488.c`, expected source SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1` from the prior tag capture.

## Question and scope

Does the unmodified GCC 15.3.0 release C compiler pass its own `gcc.dg/torture/pr116488.c` regression test on this executor's native Linux x86-64 target, using the test directives and GCC testsuite harness from the same release source?

This is one bounded G2 maintenance-history observation. The testcase checks that `b == -1` after `f(129, 128)` and aborts otherwise. A passing result applies only to the tested GCC source/build, native target, options, and execution environment. It does not establish all-target correctness, the full GCC testsuite status, a general compiler-quality rate, or completion of G2. No AIC/0.2 source, target, adopted contract, candidate format, QA threshold, or release state is changed.

## Frozen procedure

1. Retrieve `https://ftp.gnu.org/gnu/gcc/gcc-15.3.0/gcc-15.3.0.tar.xz` and the checksum published in that directory. Record final URL, HTTP status, bytes, SHA-256, and the published checksum. Stop if the archive does not match the official checksum, `gcc/BASE-VER` is not `15.3.0`, or the release testcase hash differs from the fixed value above.
2. Build the C frontend from that archive for the executor's native `x86_64-linux-gnu` target, with host GCC 14.2.0, `--enable-languages=c --disable-bootstrap --disable-multilib --disable-nls --disable-werror`, and `make -j4 all-gcc`. Use only official GCC-bundled prerequisites and user-local build tools. Do not install system packages or modify global configuration.
3. Run only `gcc/testsuite/gcc.dg/torture/pr116488.c` through the GCC 15.3.0 DejaGNU/GCC testsuite harness. Derive the exact per-file `RUNTESTFLAGS` selector from the release's own testsuite scripts; do not widen the selected testcase. Preserve the `.sum`, `.log`, exact command, test variant counts/statuses, and native runtime identity.
4. Acceptance for the test observation is a complete selected-test run with no unexpected `FAIL` or `XPASS`; record every `PASS`, `FAIL`, `UNSUPPORTED`, `UNRESOLVED`, or `UNTESTED` result verbatim. Build/configuration/test-harness failure is `NOT_RUN` for the testcase, not a pass. Do not rerun altered source or change flags to turn a failure into success.
5. Keep source, build, tool, and capture outputs under `/tmp/aic-gcc15-pr116488-2026-11-13/`. Save only provenance, hashes, commands, bounded result data, and necessary test log excerpts in the repository.

## User-local test toolchain amendment

The executor has `mawk` and Tcl 8.6.16 runtime but no GNU awk, Expect, or DejaGNU; Tcl development headers/config are also absent. Before retrieving auxiliary tool sources, freeze these upstream inputs for a user-owned prefix: GNU gawk 5.3.0 (`https://ftp.gnu.org/gnu/gawk/gawk-5.3.0.tar.xz`), Tcl 8.6.16 (`https://prdownloads.sourceforge.net/tcl/tcl8.6.16-src.tar.gz`), Expect 5.45.4 (`https://sourceforge.net/projects/expect/files/Expect/5.45.4/expect5.45.4.tar.gz/download`), and GNU DejaGnu 1.6.3 (`https://ftp.gnu.org/gnu/dejagnu/dejagnu-1.6.3.tar.gz`). Record published checksums and retrieved byte hashes before building. If a fixed source/checksum is unavailable or any tool cannot be built without system changes, stop the testsuite attempt and record the blocker; do not substitute versions after observing test results.

## Environment and boundaries

Known before preregistration: Linux `x86_64`, kernel `6.18.57`; host GCC/G++ `14.2.0`; GNU Binutils `2.44`; 5 CPUs, 17 GiB RAM, 27 GiB free disk; no `rustc`/`cargo`. No system package installer or root access is available, so required build/test utilities must be built or unpacked in a user-owned temporary prefix from fixed public sources; if that is not feasible, stop and record the exact blocker. Repository code tests are outside this record and remain `NOT_RUN`.
