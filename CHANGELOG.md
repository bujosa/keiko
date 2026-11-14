# Changelog

## 0.1.0

First release: Keiko, the launcher for the dojo apps.

- The catalog of dojos (PodDojo first), in a grid or a list, sorted by name, release date or status,
  with search.
- Install and update from GitHub releases: the build for this Mac, verified against the release's
  SHA-256 checksums, the bundle identifier and the code signature, installed in `~/Applications`;
  the previous version is kept aside until the new one is in place.
- Belts for status (white: not installed, yellow: update available, black: up to date) and
  "Update N apps".
- Each installed dojo's progress on its card (PodDojo: lessons done and the next lesson).
- Open, show in Finder, release notes, move to Trash.
- Private repositories through the GitHub CLI's token; public ones need no account.
- Dark and light themes following macOS.
