# Changelog

## 0.2.0

A more beautiful Keiko.

- **Continue your practice**: the top of the page picks up where you left off, with the next lesson,
  a progress ring and Resume; before the first dojo, a welcome with the belt as a watermark.
- **Your next dojo**: a card (or row) at the end of the catalog that links to the guide for adding one.
- Status badges carry a small knotted belt in the belt's colour; section headings with a hairline.
- Cards in a row share one height, with space between them.
- A crisp painted mark in the top bar (the vermilion seal with a knotted belt).
- The top bar moves the window; a double click zooms it.

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
