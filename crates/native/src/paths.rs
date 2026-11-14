//! Where Keiko keeps things. `KEIKO_HOME` moves the data and cache (tests, agents);
//! `KEIKO_APPS_DIR` moves where new apps are installed.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    /// Settings and the release cache.
    pub data: PathBuf,
    /// Downloads in progress.
    pub cache: PathBuf,
    /// Where Keiko installs apps (`~/Applications`: no administrator password needed).
    pub apps: PathBuf,
    /// Folders searched for installed apps, in order.
    pub search: Vec<PathBuf>,
}

impl Paths {
    pub fn detect() -> Result<Paths, String> {
        let home = std::env::var_os("HOME").map(PathBuf::from).ok_or("HOME is not set")?;
        let (data, cache) = match std::env::var_os("KEIKO_HOME") {
            Some(h) => (PathBuf::from(&h), PathBuf::from(h).join("cache")),
            None => (home.join("Library/Application Support/Keiko"), home.join("Library/Caches/Keiko")),
        };
        let apps = std::env::var_os("KEIKO_APPS_DIR").map(PathBuf::from).unwrap_or_else(|| home.join("Applications"));
        let mut search = vec![apps.clone()];
        if std::env::var_os("KEIKO_APPS_DIR").is_none() {
            search.push(PathBuf::from("/Applications"));
        }
        Ok(Paths { data, cache, apps, search })
    }

    pub fn for_tests(root: &std::path::Path) -> Paths {
        Paths {
            data: root.join("data"),
            cache: root.join("cache"),
            apps: root.join("Applications"),
            search: vec![root.join("Applications")],
        }
    }
}
