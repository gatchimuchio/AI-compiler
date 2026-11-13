# GCC PR 116488 reporter source reproduction — B62

- Record ID: `G2-HISTORY-2026-11-13-10`
- Preregistration commit: `d655e0bec1c2af49634bfec9fcd105a95142b352`
- Pre-run identity commit: `fd978e2228490ec6a9ffee6a18ffd752c662eb75`
- Result: **PASS** for the preregistered output on all four cases. The reporter's flag-dependent output difference was **not reproduced** with these two current compiler builds.
- AICompiler source, compiler contract, candidate adoption, target, and QA thresholds were not changed.

## Source and fixed question

The input was the reporter's 14-line `small.c` from the 2024-08-26 GCC bug mailing-list message ([primary archive](https://gcc.gnu.org/pipermail/gcc-bugs/2024-August/878977.html)). The archived response was captured with HTTP 200 on 2026-11-13: 5,038 bytes, SHA-256 `7f9c95825ed37234ae4d7d877a278609909a131470e716fcaa4706fda56d6f0d`. The extracted source is 236 bytes, SHA-256 `77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2`. Extraction provenance and the unchanged external source are identified in [source capture metadata](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/report-source-capture.json).

The preregistered question compared `-O3` with `-fno-forward-propagate -O3` under Debian GCC 14.2.0-19 and a separately built GCC 15.2.0 release compiler. Before the run, the expected output for every case was fixed as the byte sequence `0\n-1\n`. The original report says its GCC 15.0.0 experimental build printed `0\n-1\n` without the flag and `0\n0\n` with the flag, and says the issue did not reproduce on GCC 14.2 or earlier. That historical compiler build was not reconstructed here.

## Compilers and execution conditions

| Compiler | Build identity | Target | Driver SHA-256 |
|---|---|---|---|
| Debian GCC 14.2.0-19 | installed Debian package build | `x86_64-linux-gnu` | `a23ecab8ff08f09ad8c80602c2c5df7f49e09c25905cb8975902e101bf72635f` |
| GCC 15.2.0 | B60 build from official `gcc-15.2.0.tar.xz`; archive SHA-256 `438fd996826b0c82485a29da03a72d71d6e3541a83ec702df4271f6fe025d24e`; peeled source commit `5115c7e447fc07457443df874bf57840e8316d5f` | `x86_64-linux-gnu` | `d11bd49769c79c84969134620c9c7607fb8cf53d0be074e8e012b9cc47c4b297` |

Both compiler/target pairs ran natively on Linux x86-64. GCC 15.2.0 used its build prefix via `-B`; binary, `cc1`, runtime, host, assembler, linker, libc, timeout, environment, and command identities are in the [pretest record](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/pretest-identity.json). The pretest identity was committed before any testcase compile/run. The runner and command matrix are fixed in the [preregistration](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/preregistration.md).

Each of these four exact compiler invocations ran once in a new directory containing a byte-identical `small.c` copy:

```text
/usr/bin/gcc-14 -O3 small.c
/usr/bin/gcc-14 -fno-forward-propagate -O3 small.c
/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -O3 small.c
/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/xgcc -B/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0/gcc/ -fno-forward-propagate -O3 small.c
```

For each case the compile process and native `./a.out` consumer exited 0 without timeout. Compiler stdout and stderr were empty. Each consumer produced exactly 5 bytes, `0\n-1\n`; its SHA-256 was `c7b4ea6821495a8eb0ebc912a385717bcbc0fde124e8f92195d96e25455db0af`. Full per-process argv, environment, UTC times, exit states, and stdout/stderr hashes are in [results.json](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/results.json).

| Case | `a.out` bytes | `a.out` SHA-256 | `objdump -drwC -Mintel` stdout SHA-256 | Consumer stdout |
|---|---:|---|---|---|
| Debian GCC 14.2.0-19, `-O3` | 16,104 | `5b7e14aecfb756cfd91066b5cc7458992a2cf3a3e16de31eb4291bdc2778fe90` | `a588d0f559d176f1cba27ad18706611b865cb96a50d31153534fe62280b76dc1` | `0\n-1\n` |
| Debian GCC 14.2.0-19, `-fno-forward-propagate -O3` | 16,104 | `4abf5c4c1d124ca014b3136ee168dde2cab27870e20b76dbffdb80fcbe56ea5a` | `fec1949ddb245b52bd60b55c22dfa2b4900676d6dffe222665904509a9f24525` | `0\n-1\n` |
| GCC 15.2.0, `-O3` | 15,840 | `5ac0750840ad00abda0471fc9113fe4b5d87cbedfd95ddd7a64879cefa8ea61b` | `a59e72b8bfb766e7b113edc06c9b8a8518a8a141edd482e31d82e848129a522c` | `0\n-1\n` |
| GCC 15.2.0, `-fno-forward-propagate -O3` | 15,840 | `5ac0750840ad00abda0471fc9113fe4b5d87cbedfd95ddd7a64879cefa8ea61b` | `b3b5e0f6fdd401bf4d0e12abb782add1e1f888a281a8e8fed9ed6f85f309ba29` | `0\n-1\n` |

The GCC 15.2.0 executables are byte-identical across the two flag cases. The GCC 14.2.0 executables differ, but both produce the preregistered output. In the disassemblies, all four `f` implementations contain an arithmetic right shift (`sar ...,0x10`) and sign extension of the byte-derived value (`movsx`); GCC 14.2 changes register allocation and its function prologue with the flag while retaining that observed result path. The GCC 15.2.0 disassemblies contain identical instructions; after replacing only the differing absolute binary path in each output header, both captures have SHA-256 `389e787fd8a4b20f73880ad627a56569343df85430a2e0d56dad9265d5b7bb81`. Full read-only disassemblies, tool version, command and artifact hashes are retained under the external root listed in the [capture manifest](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/capture-manifest.json), with a summary in the [machine review](../../experiments/g2-gcc-pr116488-reporter-reproducer-2026-11-13/observations/postrun-machine-review.json).

## Interpretation and limits

- **Observed:** the original reporter source compiled and ran successfully under these two builds; all four outputs were `0\n-1\n`. The GCC 15.0.0 experimental flag-dependent difference described in the report did not appear in the GCC 15.2.0 release build or this Debian GCC 14.2.0-19 build.
- **Independent oracle: NOT_RUN.** Expected output was preregistered from the report and its stated non-reproducing fixed-build behavior; no separately implemented semantic oracle or third-party consumer was used. B64's read-only N1570 audit found that plain `char`, out-of-range signed conversion, and negative signed right shift leave implementation-defined choices, so C11 alone does not determine a unique expected stdout ([B64 audit](G2_gcc_pr116488_c11_semantics_2026-11-13.md)).
- The source declares plain `char`. On this x86-64 environment it is signed, as also reflected by the emitted sign-extension instructions. The negative signed right shift in the source has implementation-defined C behavior; this observation does not establish the same result on other targets, compiler defaults, or language modes.
- This is one reporter testcase, two compiler builds, one target, and one execution per condition. It does not recreate the original 2024 GCC 15.0.0 experimental build, establish behavior for other builds/targets/flags, or test the complete GCC suite. File-size differences are not performance measurements and are not ranked.
- The result adds one fixed reproduction observation to G2's compiler-to-binary-to-consumer record. It does not complete G2's required independent-oracle, broader compatibility, or accumulated operational-history evidence. AICompiler repository tests were **NOT_RUN**.

The external B62 root contains 100 regular-file paths totaling 146,989 bytes, including the source archive capture, identity logs, run inputs/outputs, four executables, and read-only disassemblies. Per-file hashes and the five-class asset inventory are recorded in the repository capture manifest and [asset inventory](G3_資産棚卸_2026-11-13.json).
