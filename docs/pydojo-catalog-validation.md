# Three-dojo catalog validation · 0.3.7

PyDojo joins the existing PodDojo and TypeDojo entries after its signed public release is available. The catalog uses its stable bundle identifier, architecture-specific ZIP pattern and SHA256SUMS.txt. Its original icon and mint accent match the app.

`cargo xtask ci` passed formatting, Clippy and all existing tests. The anonymous network suite (`KEIKO_NO_GH=1`, token variables unset) passed four integration tests: PodDojo, TypeDojo, PyDojo and launcher self-update. Each dojo test downloads a public release into owned temporary folders, checks the checksum/signature/bundle identity, replaces the bundle, refuses an invalid asset, uninstalls, restores and retains sentinel test data. Ordinary Applications, Trash and learner profiles are not used.

Manual native review used both KEIKO_HOME and KEIKO_APPS_DIR overrides. All three available cards were aligned and dark/desaturated. Installing the public PyDojo asset changed its card to color, Up to date and Open. The app opened from the launcher into an isolated learner profile. The confirmation explained saved-data retention; uninstall returned the card to Available and moved the bundle into isolated Trash. No automated presentation tests were added.

The launcher remains ad-hoc signed and Apple Silicon only. The catalog does not waive each dojo's own runtime prerequisites or turn its educational execution guards into a security sandbox. No catalog or settings schema migration is needed.
