# Adding a dojo

A dojo joins Keiko with a pull request that adds an `[[apps]]` entry to `catalog.toml` (and its
icon to `assets/apps/<id>.png`, wired in `apps/keiko/src/app.rs › app_png`).

## What the dojo must provide

1. **GitHub releases** tagged `vX.Y.Z` (or `X.Y.Z`). Drafts and prereleases are ignored.
2. **A zipped macOS app** per architecture, named by the entry's `asset` pattern, for example
   `PodDojo-v{version}-macos-{arch}.zip` with `{arch}` = `arm64` or `x86_64`. The zip contains
   `<mac_bundle_name>.app` at its top level (`ditto -c -k --keepParent` produces this).
3. **A checksums file** in the same release (`SHA256SUMS.txt` by default) in `shasum -a 256`
   format, listing the zip. Keiko refuses a release without it.
4. **A stable bundle identifier** (`CFBundleIdentifier`) and a version in
   `CFBundleShortVersionString`. Keiko only replaces an installed app with the same identifier.
5. **A valid code signature** (ad-hoc is accepted; `codesign --verify --deep --strict` must pass).

## Naming

Dojos are named `<Topic>Dojo` (PodDojo, RustDojo…): Keiko draws the topic in the text colour and
"Dojo" in the entry's `color`.
