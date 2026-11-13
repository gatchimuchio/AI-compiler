# GCC PR 116488 reporter source reproduction — B62

- Record ID: `G2-HISTORY-2026-11-13-10`
- Result: **PASS** for the preregistered expected output; the reported flag-dependent output difference was not reproduced in the two current compiler builds.
- Four native Linux x86-64 compile/run cases completed once each: Debian GCC 14.2.0-19 and the B60 GCC 15.2.0 release build, with `-O3` and with `-fno-forward-propagate -O3`.
- All four executables exited 0 and printed exactly `0\n-1\n` (5 bytes). Actual binaries and read-only disassemblies were hashed after execution.

## Records

- Frozen source, question, commands, expectations, and stop rules: [preregistration](preregistration.md)
- Compiler, host, runtime, source, and runner identities: [pretest identity](observations/pretest-identity.json)
- Run stdout/stderr, exit statuses, times, arguments, and environments: [results](observations/results.json)
- Executable hashes and `objdump` capture hashes: [post-run machine review](observations/postrun-machine-review.json)
- Hash/size manifest for the 100 retained external files (146,989 bytes): [capture manifest](observations/capture-manifest.json)
- Interpretation, exact outputs, transformation observations, and limits: [B62 evidence](../../docs/evidence/G2_gcc_pr116488_reporter_reproduction_2026-11-13.md)

Raw source capture, four `a.out` files, runtime outputs, and complete disassemblies are retained under `/tmp/aic-g2-gcc-pr116488-reporter-reproducer-2026-11-13/`. The reporter's plain-`char` source and the original GCC 15.0.0 experimental build have limitations described in the evidence. AICompiler project tests were **NOT_RUN**.
