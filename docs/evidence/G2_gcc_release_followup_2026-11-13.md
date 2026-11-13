# GCC PR 116488 stable release follow-up

- Record ID: `G2-HISTORY-2026-11-13-05`
- Survey date: 2026-11-13 UTC
- Repository baseline HEAD before preregistration: `d61fc4f7145c45aef6edab99bc221d1f12c281bb`
- Preregistration commit: `85c64f5814cb16f5b4a6fe6675f8ada9d01dba88`
- Existing incident: GCC PR 116488; fix commit `36e91df7716d34aa5694533837551593ec28f22b`; testcase portability correction `8df549fc2c0297970cb3bcadba7929088af96522`.
- Question: do GCC 15.1.0, 15.2.0, and 15.3.0 stable source tags contain the fix behavior and corrected signed-char testcase?
- Result: **source-level presence observed in all three tags**. This is a bounded maintenance-history record. It does not establish that GCC was built or that its tests passed in this work unit.
- Adoption: no change to AIC/0.2, the proposed G0–G6 plan, compiler/toolchain selection, target, QA state, or release status.

## Method and version identity

The preregistered capture ran from `2026-11-13T07:24:24Z` to `2026-11-13T07:27:08Z`. `git ls-remote` queried the three release refs on both GCC's official Git remote (`https://gcc.gnu.org/git/gcc.git`) and the `gcc-mirror/gcc` GitHub mirror. The captured ref values matched byte-for-byte. A B59 follow-up query later confirmed that these values are annotated tag-object IDs, not peeled commit IDs, and captured the peeled IDs from both remotes; see the identity correction below and the B59 experiment. Each release file was downloaded from the mirror's release-tag URL, with the selected tag object and each file hash recorded. The GitHub Contents API blob ID computed from each raw file matched the API-reported blob ID.

| Stable release | Official date | Tag object ID | Peeled commit ID | `gcc/ext-dce.cc` | `pr116488.c` |
|---|---|---|---|---|---|
| 15.1.0 | 2025-04-25 | `f33bb98ad1669db947ef9351939d242e4a88542b` | `1b306039ac49f8ad91ca71d3de3150a3c9fa792a` | 37,425 bytes; SHA-256 `56ad15e68cce43c7f6f9ad4db81d464ffea44076c20eb7a202820ae0230cdd1e`; Git blob `a03439501419df9fccf712383040b2b2ea556c9f` | 399 bytes; SHA-256 `10182dfccf8dec109189e28ef12484371c6b6c594dcdb76ebc468c7d1d0a6aa1`; Git blob `b2af84fef43040e7d9651386f52930b97a70d469` |
| 15.2.0 | 2025-08-08 | `dcd428f94ffb464418f996ffb70dfa398f5caa3f` | `5115c7e447fc07457443df874bf57840e8316d5f` | 40,217 bytes; SHA-256 `f6b40d2ec541e522d98d9c9801c7427c6a1c0fc6f74dddbdc976b0036a07d76f`; Git blob `67ec92a42878ac1ef1e133cd57a163f1ad4a3159` | Same bytes and Git blob as 15.1.0 |
| 15.3.0 | 2026-06-12 | `6d02d02c0b733e4934b8e1e497d90612116de9af` | `4db0e8df15bef836558857c291c323add11d035c` | Same bytes and Git blob as 15.2.0 | Same bytes and Git blob as 15.1.0 |

The official GCC 15 series page calls 15.1.0 the release starting the stable 15 series and lists the dates above. The corresponding `gcc-announce` release messages were captured separately. The 15.2 and 15.3 source snapshots keep the same testcase blob; `ext-dce.cc` is byte-identical at 15.2 and 15.3.

## Fix behavior in the stable source

The fix commit patch identifies two changes: `carry_backpropagate` handles `SIGN_EXTEND` by keeping the source sign bit live and recursing into the operand, and also recurses through `ZERO_EXTEND`; `ext_dce_process_uses` performs this backpropagation before the subsequent optimization/extension-stripping steps.

The GCC 15.1.0 source contains these behaviors in the matching function and call sequence: [`carry_backpropagate` SIGN/ZERO extension handling](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15.1.0/gcc/ext-dce.cc#L564-L585), and the backpropagation call before extension stripping and the optimization point ([lines 697–723](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15.1.0/gcc/ext-dce.cc#L697-L723)). At 15.2.0 the corresponding locations are [lines 570–591](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15.2.0/gcc/ext-dce.cc#L570-L591) and [lines 702–728](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15.2.0/gcc/ext-dce.cc#L702-L728). The 15.3.0 `ext-dce.cc` blob is byte-identical to 15.2.0, so the same lines apply.

This is a behavior/source observation, not a claim that the release file is byte-identical to the original fix-commit file or that the release tag's ancestry was independently traversed. Later GCC changes altered surrounding source and spelling while the registered behavior remained present.

## Corrected regression testcase in each tag

The portability commit changes `char c, e;` to `signed char c, e;` after unsigned-char-defaulting targets failed the testcase. All three release-tag files contain `signed char c, e;`, the `-fno-forward-propagate` option, and the run directive. The GCC 15.1.0 test is visible at [`pr116488.c` lines 1–22](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15.1.0/gcc/testsuite/gcc.dg/torture/pr116488.c#L1-L22); the stable files also include a later `dg-require-effective-target int32plus` guard not present in the portability-commit snapshot. The testcase blob is identical across 15.1.0, 15.2.0, and 15.3.0.

The official GCC 15 changes page response did not contain the literal `116488`. That omission is not treated as evidence against inclusion: release PR lists can be incomplete, and the tag source is the direct observation for this question.

## Primary source captures

| Source | Use | Retrieved bytes / SHA-256 |
|---|---|---|
| [GCC 15 release series](https://gcc.gnu.org/gcc-15/) | Stable release dates and series status | 3,025 / `0df3bd64e458f2c503be2955a618747524de3b1067326b47b3a48b16fd4c0113` |
| [GCC 15 changes](https://gcc.gnu.org/gcc-15/changes.html) | Versioned change page; literal PR 116488 not found in captured body | 78,698 / `e5949fc3a7a2e8a2f99297964195b61baec2b8e59642e5f1e258a7ac28b93963` |
| [GCC 15.1 release announcement](https://gcc.gnu.org/pipermail/gcc-announce/2025/000185.html) | Announcement date/title | 4,538 / `4c66d6b3b85af5d27eb2a719e04a3373593119666c74f7b6747192a0b3a7aeb1` |
| [GCC 15.2 release announcement](https://gcc.gnu.org/pipermail/gcc-announce/2025/000189.html) | Announcement date/title | 2,819 / `9efa8b72ed58e6a432852c53baa3bc445e9c552961e58bda38bf156f9dd80892` |
| [GCC 15.3 release announcement](https://gcc.gnu.org/pipermail/gcc-announce/2026/000191.html) | Announcement date/title | 3,015 / `c53fa488f53dd7feb4d7659087e48d3c83768ea3592cdb9f624a063fa2e32fd7` |
| Official Sourceware refs and `gcc-mirror/gcc` mirror refs | Initial B57 exact-ref `ls-remote` outputs list identical tag-object IDs, not peeled commits; B59 follow-up captures both forms | B57 each stdout 213 / SHA-256 `4f40d4532615c15ed63fe42a5404c3fcb16f7d6d16d284a758bcc2d62d3301e3`; B59 each stdout 435 / SHA-256 `aa81ab322d59adc34908312aeaf0fb6604a9aa2a7907bb4fa86465e2346ee4cd` |
| [Fix commit patch](https://github.com/gcc-mirror/gcc/commit/36e91df7716d34aa5694533837551593ec28f22b.patch) | Registered change fingerprint | 7,605 / `1888cf3ea41ba8d392d18bab82cdbf6773fc1e899feb1fd9c776c38d4c29b877` |
| [Testcase portability commit patch](https://github.com/gcc-mirror/gcc/commit/8df549fc2c0297970cb3bcadba7929088af96522.patch) | `char` → `signed char` correction | 1,896 / `96f4082bfaf3e4903f1bc29019aff27ceb62a157c7e67a7384ed786b094b3c76` |

The complete response manifest records URLs, HTTP statuses, retrieval times, byte lengths, raw SHA-256 values, tag/file blob identities, and the raw-capture paths. It is [capture-manifest.json](../../experiments/g2-gcc-pr116488-release-followup-2026-11-13/observations/capture-manifest.json). B59's later tag-object/peeled-commit query is recorded separately in [the B59 observations](../../experiments/g2-gcc-pr116488-testsuite-15.1-2026-11-13/observations/). HTTP bodies, source snapshots, and the original B57 Git command outputs remain outside the repository at `/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/gcc-pr116488-release-followup-2026-11-13/`.

## Limits and work not run

- The original wrong-code symptoms remain the reporter's report; this project did not reproduce them.
- At the time of B57, GCC 15.1.0, 15.2.0, and 15.3.0 had not been built here, and the GCC testsuite and `pr116488.c` had not been executed. A later B58 run built GCC 15.3.0 and executed the selected test; see [the separate execution evidence](G2_gcc_pr116488_testsuite_2026-11-13.md).
- Source inclusion does not establish that every target/configuration is correct, that every distribution uses identical sources, or that no later recurrence exists.
- This one incident and the previously recorded release examples do not represent GCC's complete maintenance history or quantify compiler reliability. G2 remains incomplete.

## Later execution record

B58 recorded 16 expected passes for GCC 15.3.0's `pr116488.c` test on native Linux x86-64. The source URL deviation, test command, hashes, and limits are in [the execution evidence](G2_gcc_pr116488_testsuite_2026-11-13.md) and [the B58 experiment](../../experiments/g2-gcc-pr116488-testsuite-2026-11-13/).

## Release-tag identity correction

B57's `ls-remote` outputs contained annotated tag-object IDs and were initially labeled as commit IDs. B59 queried each tag ref together with its peeled `^{}` commit on both remotes. Both outputs matched exactly; the corrected object/commit pairs are shown in the table above. This correction does not alter the source-file hashes/blob IDs or the source-presence finding. B58's GCC 15.3.0 result is likewise relabeled as tag object `6d02d02c0b733e4934b8e1e497d90612116de9af`, peeled commit `4db0e8df15bef836558857c291c323add11d035c`; its archive checksum, `BASE-VER`, testcase hash and test statuses are unchanged.
