# GCC 15.3.0 PR 116488 regression execution

This experiment extends the stable-source-only observation in [B57](../../docs/evidence/G2_gcc_release_followup_2026-11-13.md) with one preregistered attempt to run the exact GCC 15.3.0 testsuite regression. Scope, source identity, build configuration, pass criteria, and stop conditions are fixed in [preregistration.md](preregistration.md), including a pre-execution amendment to build the matching `libgcc` runtime.

Current state: GCC 15.3.0 source identity, auxiliary test tools, and the C frontend build are verified. The exact PR testcase has not yet been run. The next steps are `make -j4 all-target-libgcc`, followed by `make check-gcc RUNTESTFLAGS='dg-torture.exp=pr116488.c'` from the configured GCC build directory. Results and captured-output hashes will be added after that single run.
