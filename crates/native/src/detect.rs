//! Find installed apps and read their version from `Info.plist`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use keiko_core::App;
use serde::{Deserialize, Serialize};

use crate::Paths;
use crate::run::run;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Installed {
    pub path: PathBuf,
    pub version: String,
}

impl Installed {
    pub fn semver(&self) -> Option<semver::Version> {
        keiko_core::release::parse_version(&self.version)
    }
}

/// A value from a bundle's `Info.plist`.
pub fn plist_value(app: &Path, key: &str) -> Option<String> {
    let plist = app.join("Contents/Info.plist");
    if !plist.is_file() {
        return None;
    }
    let o = run(
        Command::new("/usr/bin/plutil").args(["-extract", key, "raw", "-o", "-"]).arg(&plist),
        None,
        Duration::from_secs(5),
        4096,
    )
    .ok()?;
    let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (o.ok() && !v.is_empty()).then_some(v)
}

/// `app` installed in one of the search folders, carrying the catalog's bundle identifier.
pub fn installed(paths: &Paths, app: &App) -> Option<Installed> {
    paths.search.iter().find_map(|dir| {
        let path = dir.join(format!("{}.app", app.mac_bundle_name));
        if plist_value(&path, "CFBundleIdentifier")? != app.mac_bundle_id {
            return None;
        }
        let version = plist_value(&path, "CFBundleShortVersionString")?;
        Some(Installed { path, version })
    })
}
