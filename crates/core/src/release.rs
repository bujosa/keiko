//! GitHub releases: the API document, the asset Keiko needs and the release's checksums.

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::App;

/// The subset of `GET /repos/{repo}/releases/latest` Keiko reads.
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhAsset {
    pub name: String,
    pub size: u64,
    /// The API URL (`…/releases/assets/{id}`); works for public and private repositories.
    pub url: String,
}

/// A downloadable file of the latest release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub url: String,
}

/// What Keiko keeps about an app's latest release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Latest {
    pub version: String,
    pub tag: String,
    pub html_url: String,
    pub published_at: Option<String>,
    pub notes: String,
    /// The build for this Mac, if the release has one.
    pub asset: Option<Asset>,
    pub checksums: Option<Asset>,
}

impl Latest {
    pub fn semver(&self) -> Option<Version> {
        parse_version(&self.version)
    }
}

/// `v0.3.0`, `V0.3.0` and `0.3.0` → 0.3.0. Two-part versions (`1.2`) are read as `1.2.0`.
pub fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    Version::parse(s).ok().or_else(|| {
        let parts = s.split('.').count();
        if parts == 2 { Version::parse(&format!("{s}.0")).ok() } else { None }
    })
}

/// Read a release document for `app` on `arch`. `None` for drafts and prereleases and for tags that
/// are not versions.
pub fn latest(app: &App, json: &str, arch: &str) -> Result<Option<Latest>, String> {
    let r: Release = serde_json::from_str(json).map_err(|e| format!("release: {e}"))?;
    if r.draft || r.prerelease {
        return Ok(None);
    }
    let Some(v) = parse_version(&r.tag_name) else { return Ok(None) };
    let version = v.to_string();
    let wanted = app.asset_name(&version, arch);
    let find = |name: &str| {
        r.assets.iter().find(|a| a.name == name).map(|a| Asset {
            name: a.name.clone(),
            size: a.size,
            url: a.url.clone(),
        })
    };
    Ok(Some(Latest {
        version,
        tag: r.tag_name.clone(),
        html_url: r.html_url.clone(),
        published_at: r.published_at.clone(),
        notes: r.body.clone().unwrap_or_default(),
        asset: find(&wanted),
        checksums: find(&app.checksums),
    }))
}

/// The SHA-256 (lowercase hex) of `name` in a `shasum -a 256` / `sha256sum` file.
pub fn checksum_for(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, file) = line.trim().split_once(char::is_whitespace)?;
        let file = file.trim_start().trim_start_matches('*');
        let ok = hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit());
        (ok && file == name).then(|| hash.to_ascii_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    const DOC: &str = r#"{
      "tag_name": "v0.3.0", "html_url": "https://github.com/bujosa/poddojo/releases/tag/v0.3.0",
      "published_at": "2026-10-28T20:13:36Z", "body": "notes", "draft": false, "prerelease": false,
      "assets": [
        {"name": "PodDojo-v0.3.0-macos-arm64.zip", "size": 9000000, "url": "https://api.github.com/repos/bujosa/poddojo/releases/assets/1"},
        {"name": "SHA256SUMS.txt", "size": 97, "url": "https://api.github.com/repos/bujosa/poddojo/releases/assets/2"}
      ]
    }"#;

    fn app() -> App {
        Catalog::bundled().unwrap().apps[0].clone()
    }

    #[test]
    fn reads_the_release_for_this_mac() {
        let l = latest(&app(), DOC, "arm64").unwrap().unwrap();
        assert_eq!(l.version, "0.3.0");
        assert_eq!(l.asset.unwrap().size, 9_000_000);
        assert_eq!(l.checksums.unwrap().name, "SHA256SUMS.txt");
    }

    #[test]
    fn no_asset_for_other_architectures() {
        let l = latest(&app(), DOC, "x86_64").unwrap().unwrap();
        assert!(l.asset.is_none());
    }

    #[test]
    fn skips_prereleases_and_odd_tags() {
        let pre = DOC.replace("\"prerelease\": false", "\"prerelease\": true");
        assert!(latest(&app(), &pre, "arm64").unwrap().is_none());
        let odd = DOC.replace("v0.3.0\", \"html_url", "nightly\", \"html_url");
        assert!(latest(&app(), &odd, "arm64").unwrap().is_none());
    }

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("v1.2").unwrap().to_string(), "1.2.0");
        assert!(parse_version("V0.10.1").unwrap() > parse_version("0.9.9").unwrap());
        assert!(parse_version("latest").is_none());
    }

    #[test]
    fn finds_checksums() {
        let h = "a".repeat(64);
        let sums = format!("{h}  PodDojo-v0.3.0-macos-arm64.zip\n{}  other.zip\n", "b".repeat(64));
        assert_eq!(checksum_for(&sums, "PodDojo-v0.3.0-macos-arm64.zip"), Some(h.clone()));
        assert_eq!(checksum_for(&format!("{h} *x.zip"), "x.zip"), Some(h));
        assert_eq!(checksum_for("abc  x.zip", "x.zip"), None);
        assert_eq!(checksum_for(&sums, "missing.zip"), None);
    }
}
