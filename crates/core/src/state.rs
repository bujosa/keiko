//! Which belt an app wears: not installed, up to date, update available…

use semver::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    /// A newer release than the installed one.
    UpdateAvailable,
    /// Installed and current (or newer than the latest release).
    UpToDate,
    /// Installed, the latest release is not known yet (offline, first check pending).
    Installed,
    /// Not installed and there is a build for this Mac.
    NotInstalled,
    /// Not installed and the latest release has no build for this Mac (or no release yet).
    Unavailable,
}

impl State {
    pub fn of(installed: Option<&Version>, latest: Option<&Version>, has_asset: bool) -> State {
        match (installed, latest) {
            (Some(i), Some(l)) if l > i && has_asset => State::UpdateAvailable,
            (Some(_), Some(_)) => State::UpToDate,
            (Some(_), None) => State::Installed,
            (None, Some(_)) if has_asset => State::NotInstalled,
            (None, _) => State::Unavailable,
        }
    }

    pub fn installed(self) -> bool {
        matches!(self, State::UpdateAvailable | State::UpToDate | State::Installed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn belts() {
        assert_eq!(State::of(Some(&v("0.2.1")), Some(&v("0.3.0")), true), State::UpdateAvailable);
        assert_eq!(State::of(Some(&v("0.2.1")), Some(&v("0.3.0")), false), State::UpToDate);
        assert_eq!(State::of(Some(&v("0.3.0")), Some(&v("0.3.0")), true), State::UpToDate);
        assert_eq!(State::of(Some(&v("0.4.0")), Some(&v("0.3.0")), true), State::UpToDate);
        assert_eq!(State::of(Some(&v("0.3.0")), None, false), State::Installed);
        assert_eq!(State::of(None, Some(&v("0.3.0")), true), State::NotInstalled);
        assert_eq!(State::of(None, Some(&v("0.3.0")), false), State::Unavailable);
        assert_eq!(State::of(None, None, false), State::Unavailable);
    }
}
