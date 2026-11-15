# Contributing to Keiko

Keiko is a native Rust/egui macOS launcher. Contributions to the launcher, documentation, accessibility and new dojo integrations are welcome.

## Local setup

Install Git and Rust through rustup. `rust-toolchain.toml` pins the Rust version and components. Building the macOS bundle also needs Apple’s command-line tools (`xcode-select --install`). Network access is needed to fetch Cargo dependencies and public releases; public dojo installs need no GitHub account or token.

Clone your fork, then run:

```sh
export KEIKO_HOME="$(mktemp -d -t keiko-contrib-data)"
export KEIKO_APPS_DIR="$(mktemp -d -t keiko-contrib-apps)"
cargo run -p keiko --release
```

Keep both overrides set during review. They isolate settings/cache, app detection/installation and a private test Trash at `$KEIKO_HOME/Trash`. With normal settings Keiko uses the Mac’s `~/.Trash`. Never uninstall your real dojos to test the feature.

## Map of the code

- `crates/core`: pure catalog validation, versions, releases and states.
- `crates/native`: filesystem, GitHub/curl, bundle checks and application actions.
- `apps/keiko`: native presentation; filesystem/network work belongs on background jobs.
- `catalog.toml`: bundled catalog; `assets/apps`: original dojo icons.

## Validation

```sh
cargo xtask ci
KEIKO_NO_GH=1 cargo test -p keiko-native -- --ignored
```

The first command runs format, Clippy with warnings denied and unit tests. The second requires macOS/network and installs public releases into temporary folders; it covers replacement, invalid-checksum refusal, uninstall/restore and launcher update. Remove token environment variables as well when proving anonymous access. No personal app or Trash should be used by these tests.

For a UI change, review the actual native app at wide and compact sizes, with keyboard navigation and meaningful empty/error/busy states. Do not add automated presentation/snapshot tests. Keep domain and filesystem regression tests where they prove behavior.

Before a release, also run the network tests, review the native window and build with `cargo xtask release`. Version/changelog changes go through a PR; build signed artifacts from merged source. Do not publish a download whose checksum, bundle identifier or code signature fails.

## Add a dojo

Follow [Adding a dojo](docs/adding-a-dojo.md). Publish a stable compatible release first, with its signed app ZIP and checksums; then submit the catalog entry and original icon. Try the candidate catalog using both isolated overrides and `KEIKO_CATALOG=/absolute/path/catalog.toml`.

## Uninstall behavior

See [Uninstall and restore](docs/uninstalling.md). Keep identifier checks, collision protection and preservation of learner data intact. If the Trash is inaccessible or on a different volume, preserve the installed app and show an actionable error rather than copying and deleting it.

## Send a contribution

1. Open an issue for a substantial change so the learning goal or behavior can be agreed first. Small fixes can go straight to a PR.
2. Fork this repository, clone your fork, add `https://github.com/bujosa/keiko.git` as `upstream`, and create a feature branch from `upstream/main`. Maintainer branches use `bujosa/<short-purpose>`.
3. Keep the change focused. Preserve existing learner data, unrelated changes and the app’s English UI. Use Conventional Commits, for example `fix: preserve saved drafts` or `docs: explain local setup`.
4. Open a PR against `bujosa/keiko:main`. Explain the problem, resulting behavior, commands/results, manual review and any limits. Include a screenshot for a visual change, using fictional data and an isolated profile.

There is no hosted CI. Maintainers verify the relevant local gates before merge; a fork does not need access to a maintainer’s machine, token or Apple account. Never commit secrets, real learner drafts, local settings, build output or private screenshots. Use original or compatibly licensed assets and update `ATTRIBUTION.md` and notices when adding dependencies or artwork.

Be respectful in issues and reviews. Review may ask for a smaller scope, stronger examples, clearer explanations or additional checks. See `AGENTS.md` for the repository’s engineering agreements; it is also useful when working without an agent.

## Good first contributions

- Reproduce a reported issue with clear steps and a fictional profile.
- Improve a confusing explanation, hint, keyboard instruction or diagnostic.
- Fix a narrow accessibility or layout problem and include native visual evidence.
- Review an existing issue or roadmap item before starting a large feature.

Contributions are credited in the commit history and [GitHub contributors graph](https://github.com/bujosa/keiko/graphs/contributors).
