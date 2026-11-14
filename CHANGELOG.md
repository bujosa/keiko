# Changelog

## 0.3.2

A catalog that scales.

- **Tabs**: All, Installed, Updates and Available, each with its count.
- **Compact cards**: the version and the belt share the card's header; a smaller icon, two lines of
  description at most, lower buttons. Up to six columns (five at 1400 px).
- **Compact list**: shorter rows, so a screen holds a dozen dojos.
- `KEIKO_CATALOG=<file>` loads a catalog from disk, to try a new dojo before it ships.

## 0.3.1

A professional identity.

- **New icon**: the kanji 稽 (*keiko*, practice) on a vermilion hanko seal, on Apple's icon grid with
  a soft shadow. It reads at every size, from the menu bar to the Dock; the top bar and the update
  banner use the same artwork.
- Dojo icons are shown without Apple's margin, so they fill their place on the card.
- Cards read in order: the description, then one line of facts (size and release date). Status
  badges carry a solid belt-coloured band; "BY KEIKO" is a quiet outline.

## 0.3.0

Keiko does one job well: install the dojos, keep them up to date, open them.

- **Notifications**: Keiko checks for new releases every hour while it is open and sends a macOS
  notification once for each new version.
- **Keiko updates itself**: a banner offers the new version with *Update and restart*, verified like
  any dojo (SHA-256, bundle identifier, code signature).
- A cleaner catalog: the practice hero, the progress on the cards and the next-dojo placeholder are
  gone.

## 0.2.0

A more beautiful Keiko.

- **Continue your practice**: the top of the page picks up where you left off, with the next lesson,
  a progress ring and Resume; before the first dojo, a welcome with the belt as a watermark.
- **Your next dojo**: a card (or row) at the end of the catalog that links to the guide for adding one.
- Status badges carry a small knotted belt in the belt's colour; section headings with a hairline.
- Cards in a row share one height, with space between them.
- A new app icon and a crisp painted mark in the top bar: the vermilion seal with a knotted belt.
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
