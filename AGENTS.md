# Keiko — instructions for agents

Keiko is the launcher for the dojo apps: an ultralight, pure-Rust (egui) macOS app that lists every
dojo in `catalog.toml`, installs the right build from its GitHub release after verifying it, tells
when an update is out (hourly checks, one notification per version), opens installed dojos and
updates itself. Everything is in English. PodDojo (`bujosa/poddojo`) is the first dojo.

## Layout
- `catalog.toml` — the dojos (embedded at build time). Adding one: `docs/adding-a-dojo.md`.
- `crates/core` — catalog + validation, GitHub release model, belts (`State`), formatting.
  Pure: no processes, no network, no file system.
- `crates/native` — `github` (curl), `install` (verify, place, uninstall), `detect` (plutil),
  `launch` (open, relaunch), `notify` (macOS notifications), `store` (settings, release cache,
  notified versions), `run` (timeouts).
- `apps/keiko` — the egui window (thin: every side effect goes through `jobs` on a thread).
- `xtask` — `cargo xtask ci | icon | bundle [--install] | release`.

## Non-negotiables

### Safety: the user's Mac is real
- Install only what passed every check: SHA-256 from the release's checksums file, the catalog's
  bundle identifier, `codesign --verify --deep --strict`. Never weaken or skip a check.
- Only ever replace or trash a bundle that carries the catalog's identifier. Uninstall moves to the
  Trash; never `rm` anything outside Keiko's data and cache folders.
- The GitHub token is only sent to `https://api.github.com/`, on curl's stdin, never logged.
- Catalog entries are validated (`App::validate`): ids, repositories and file names cannot contain
  paths.

### Never crash
- No `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, `unimplemented!` in shipped code
  (workspace clippy lints deny them; tests may use them). `unsafe_code = "forbid"`.
- The UI never blocks: commands and network go through `apps/keiko/src/jobs.rs`.

### Ultralight
- Pure Rust, egui + glow, no web view, no async runtime, no HTTP/TLS crates: drive `curl`, `ditto`,
  `plutil`, `codesign`, `open`.
- Release binaries must not contain the builder's home folder (`cargo xtask bundle` checks).

### Assets
- Every image is original work (MIT OR Apache-2.0) or OFL fonts, listed in `ATTRIBUTION.md`.
- The icon has two drawings: `keiko.svg` (128 px and up) and `keiko-small.svg` (16 to 64 px, top
  bar). Change both, then `cargo xtask icon`.
- No emojis in the UI; icons are painted (`widgets::paint_icon`).

## Quality gates (before every commit)
No hosted CI. `cargo xtask ci` = fmt check, clippy `-D warnings`, tests. Before a release also run
`cargo test -p keiko-native -- --ignored` (a real install of PodDojo into a temp folder) and look at
the window (`KEIKO_SCREENSHOT=<png>`). Conventional Commits.

## Running
- App: `cargo run -p keiko --release`.
- `KEIKO_CATALOG=<file>` loads a catalog from disk (try a new dojo; screenshots of a full catalog).
- `KEIKO_HOME=<dir>` isolates settings and cache; `KEIKO_APPS_DIR=<dir>` installs and detects there
  only (use both in tests and screenshots, never touch the user's ~/Applications from tests).
