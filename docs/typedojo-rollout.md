# TypeDojo catalog rollout — v0.3.4

The candidate catalog adds TypeDojo and its original icon. It carries the reviewed card alignment changes from `bujosa/keiko-card-symmetry`; that worktree is untouched.

## Baseline before rollout

The installed Keiko 0.3.3 embeds the old catalog and only lists PodDojo. TypeDojo 0.1.0 is installed locally but has no accessible GitHub repository/release at `bujosa/typedojo` for this session. The candidate is not installed over the real launcher. Do not publish this catalog until its download is live.

Both existing repositories, `bujosa/keiko` and `bujosa/poddojo`, are private. David authorized making Keiko, PodDojo and TypeDojo public on 2026-11-15, with TypeDojo publication and Keiko merge/release rollout.

## Verified TypeDojo artifact

- Version: 0.1.0; bundle identifier: `dev.bujosa.typedojo`.
- Archive: `TypeDojo-v0.1.0-macos-arm64.zip` (5,191,725 bytes).
- SHA-256: `2ab4bbf0fc8fbb47e47e1470604d12b68b95f2db715a1e70e25f1c5846c43031`.
- Signature: `codesign --verify --deep --strict` passed; ad-hoc signed, not notarized.
- ZIP root and bundle identifier verified; checksums asset matches.
- Published source: [TypeDojo PR #1](https://github.com/bujosa/typedojo/pull/1), merged as `b6c2afacbdcb3b347c88b8a66127fa4ec73853de`. Signed artifacts were rebuilt from that merged source in the isolated `typedojo-rollout` worktree. Its `docs/e2e-validation-2026-11-15.md` records the completed native TypeDojo checks and remaining ecosystem limits.
- Initial supported download: macOS Apple Silicon. Node.js 22.13+ and TypeScript 5.9+ are required separately. This initial course is not a complete professional qualification.

## Release sequence

1. Review and commit TypeDojo on a `bujosa/` branch, run its required gates and publish its public repository through a PR. Preserve all other worktrees.
2. Publish the verified v0.1.0 source tag, ZIP and `SHA256SUMS.txt` to that repository.
3. Run the real Keiko install into isolated `KEIKO_HOME` and `KEIKO_APPS_DIR`: Available → Install → checksum/signature verification → Installed/Open. Test a bad checksum without replacing the existing app.
4. Publish the focused Keiko catalog/icon change through a PR, preserving the layout correction. Build and version the launcher release after its required gates.
5. Update local Keiko with a verified matching bundle and backup. Confirm normal PodDojo and TypeDojo cards without QA names. Preserve settings and learner progress.

## State semantics

An absent app with a compatible release shows a white belt, Not installed, and an enabled Install button. An absent app without a compatible release shows Not available and has no enabled installer. An installed local app can be opened even when its release lookup fails. Listing an app does not publish its downloads.

## Validation of this candidate

- `cargo xtask ci`: passed (format, clippy with denied warnings, 19 tests; 2 live-network tests ignored by that command).
- TypeDojo signature, archive structure, identifier, version and checksum: passed.
- First native review: with both folders isolated and no installed apps, PodDojo showed Not installed / Install; TypeDojo showed No release yet / Not available with the original icon.
- Clicked real PodDojo Install: downloaded and installed 0.3.2 into the isolated folder, then displayed Up to date / Open. Independently verified the installed identifier, version and code signature. The real learner applications were untouched.
- Available filter and list view reviewed. This exposed a confusing Available count that included missing releases. The candidate now reserves Available for installable dojos and groups other absent apps under Not available in All. The empty Updates copy now says No updates found rather than asserting all apps are current.
- Final native review: All showed Installed · 1 and Not available · 1; Available counted 0 and explained that no installable dojos remained; Updates showed No updates found. List and grid both respected the state boundary.
- Final `cargo xtask ci`: passed (format, clippy and 19 tests). `cargo xtask bundle`: passed, including home-path check and ad-hoc signature verification.
- Review app closed; real installed Keiko restored. No repository, release, PR, installed launcher or learner data changed by this preparation.

## Publication evidence — 2026-11-15

- Gitleaks 8.28.0: no findings in source/history for all three projects or in existing Keiko/PodDojo GitHub metadata. Latest distributed Keiko/PodDojo binaries contained no builder home path.
- All three repositories are PUBLIC and their unauthenticated repository APIs succeed.
- [TypeDojo v0.1.0](https://github.com/bujosa/typedojo/releases/tag/v0.1.0) is a published stable release with the arm64 ZIP and SHA256SUMS.txt; its public latest-release API returns both assets without credentials.
- All 14 TypeDojo tests passed. Original and guided contracts each passed 318 cases; all 53 explained examples and 53 playgrounds verified. Prettier checked 679 fields with no changes.
- Anonymous Keiko native tests passed: install/replacement/checksum refusal for TypeDojo and PodDojo, and launcher self-update. All three passed with GH_TOKEN, GITHUB_TOKEN and KEIKO_GITHUB_TOKEN removed and KEIKO_NO_GH=1.
- Native anonymous catalog install: with both application/data folders isolated and initially empty, the real public TypeDojo card showed 0.1.0, size/date and Install. Clicking Install downloaded and verified it, then showed Up to date / Open and the installed toast. The review app was closed afterwards.
