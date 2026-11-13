# GCC PR 116488 stable release follow-up — preregistration

- Record ID: `G2-HISTORY-2026-11-13-05`
- Baseline repository HEAD: `d61fc4f7145c45aef6edab99bc221d1f12c281bb`
- Cutoff date: 2026-11-13 UTC
- Existing incident: GCC PR 116488, trunk fix `36e91df7716d34aa5694533837551593ec28f22b`, testcase portability correction `8df549fc2c0297970cb3bcadba7929088af96522`.
- Question: whether the fix and the corrected `signed char` regression testcase are present in the GCC 15.1.0, 15.2.0, and 15.3.0 stable release source snapshots.
- Purpose and scope: add a version-bound GCC maintenance-history observation to G2. No AIC/0.2 specification, implementation, candidate adoption, target, QA status, or release state changes.

## Exploratory findings versus fixed evidence

Before this preregistration, a web search surfaced the official GCC 15.1 release announcement, the official GCC 15 series/release pages, and the official GCC 15 changes page. Those search results are discovery leads only; no retrieved response body, release tag source, or source hash from that search is treated as evidence. Definitive observations will be freshly captured from the fixed sources below after this preregistration is saved.

## Fixed sources and observations

1. Query upstream GCC Git refs for `refs/tags/releases/gcc-15.1.0`, `releases/gcc-15.2.0`, and `releases/gcc-15.3.0` at `https://gcc.gnu.org/git/gcc.git`. Record command, UTC time, and raw stdout/stderr hashes. If reachable, compare the resulting tag object/commit IDs with the GCC source mirror refs; label the mirror as a mirror.
2. Capture the official series page `https://gcc.gnu.org/gcc-15/`, the official changes page `https://gcc.gnu.org/gcc-15/changes.html`, and release announcements for 15.1, 15.2, and 15.3 from `https://gcc.gnu.org/pipermail/gcc-announce/`. Record exact URL, HTTP status, retrieval time, byte length, and SHA-256 of each response body. Announcements establish publisher-reported dates and context, not code/test inclusion.
3. For each of the three fixed tags, capture `gcc/ext-dce.cc` and `gcc/testsuite/gcc.dg/torture/pr116488.c` from the GCC source mirror's raw tag URL. Record the selected tag and immutable tag commit, source blob ID when available, byte length, SHA-256, and the specific source/test lines used to identify (a) the `SIGN_EXTEND` handling associated with the fix and (b) the corrected `signed char c, e` declarations. Compare snapshots bytewise and inspect the cited commits' diffs to make sure the fingerprint is specific to the registered changes.
4. Store response bodies and source snapshots outside Git under `/tmp/aic-g2-subwrap-compiler-compare-2026-11-13/gcc-pr116488-release-followup-2026-11-13/`. Store only manifests, hashes, source URLs/refs, and analysis in the repository.

## Interpretation and stop conditions

- A matching source snapshot establishes that these source/test contents occur in that fixed release tag. It does not show that this project built GCC, ran the testcase, independently reproduced the original report, or audited all targets/configurations.
- An absent marker in one fixed tag is a bounded observation only; it does not show absence from every branch, downstream distribution, or future release.
- Release announcements and issue status are publisher/reporter records. Bug-count totals do not become a denominator or quality rate.
- If an official ref, announcement, mirror source, or requested source path cannot be fetched or the mirror/tag identity cannot be resolved, record the failure and narrow the conclusion. Do not replace it with an unofficial summary.
- No GCC build, GCC testsuite, repository code tests, compiler implementation changes, issue/PR writes, external contact, or release operation is in scope.
