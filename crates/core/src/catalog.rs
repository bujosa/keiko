//! The catalog (`catalog.toml` at the repository root, embedded at build time).

use serde::{Deserialize, Serialize};

static BUNDLED: &str = include_str!("../../../catalog.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Catalog {
    pub apps: Vec<App>,
    /// Keiko's own release, for self-update.
    #[serde(default)]
    pub launcher: Option<App>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    /// `owner/name` on GitHub.
    pub repo: String,
    /// `PodDojo` for `PodDojo.app`.
    pub mac_bundle_name: String,
    /// The bundle identifier the installed app must carry.
    pub mac_bundle_id: String,
    /// Release asset name with `{version}` and `{arch}` placeholders.
    pub asset: String,
    #[serde(default = "default_checksums")]
    pub checksums: String,
    pub color: [u8; 3],
}

fn default_checksums() -> String {
    "SHA256SUMS.txt".into()
}

impl Catalog {
    /// The catalog that ships inside Keiko.
    pub fn bundled() -> Result<Catalog, String> {
        Catalog::parse(BUNDLED)
    }

    pub fn parse(text: &str) -> Result<Catalog, String> {
        let catalog: Catalog = toml::from_str(text).map_err(|e| format!("catalog: {e}"))?;
        let mut seen = std::collections::BTreeSet::new();
        if let Some(l) = &catalog.launcher {
            l.validate()?;
        }
        for app in &catalog.apps {
            app.validate()?;
            if !seen.insert(app.id.clone()) {
                return Err(format!("catalog: duplicate app id {}", app.id));
            }
        }
        Ok(catalog)
    }

    pub fn get(&self, id: &str) -> Option<&App> {
        self.apps.iter().find(|a| a.id == id)
    }
}

/// Lowercase letters, digits and dashes, starting with a letter.
fn safe_id(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A single path component: no separators, no `..`, no leading dot.
fn safe_name(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('.')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '{' | '}'))
}

impl App {
    pub fn validate(&self) -> Result<(), String> {
        let err = |what: &str| Err(format!("catalog: app {:?}: {what}", self.id));
        if !safe_id(&self.id) {
            return err("id must be lowercase letters, digits and dashes");
        }
        let parts: Vec<&str> = self.repo.split('/').collect();
        let gh =
            |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
        if parts.len() != 2 || !parts.iter().all(|p| gh(p)) || parts.iter().any(|p| p.starts_with('.')) {
            return err("repo must be owner/name");
        }
        if !safe_name(&self.mac_bundle_name) || self.mac_bundle_name.contains(['{', '}']) {
            return err("mac_bundle_name must be a plain name");
        }
        if !self.asset.ends_with(".zip") || !safe_name(&self.asset) {
            return err("asset must be a .zip file name");
        }
        if !safe_name(&self.checksums) || self.checksums.contains(['{', '}']) {
            return err("checksums must be a plain file name");
        }
        if self.mac_bundle_id.is_empty() || !self.mac_bundle_id.contains('.') {
            return err("mac_bundle_id must be a reverse-DNS identifier");
        }
        Ok(())
    }

    /// The release asset this Mac needs for `version` (without the leading `v`).
    pub fn asset_name(&self, version: &str, arch: &str) -> String {
        self.asset.replace("{version}", version).replace("{arch}", arch)
    }

    pub fn github_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }

    /// `PodDojo` → (`Pod`, `Dojo`): the name drawn in two tones on the card.
    pub fn name_parts(&self) -> (&str, &str) {
        for suffix in ["Dojo", "dojo"] {
            if let Some(head) = self.name.strip_suffix(suffix)
                && !head.is_empty()
            {
                return (head, &self.name[head.len()..]);
            }
        }
        (&self.name, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_is_valid() {
        let c = Catalog::bundled().unwrap();
        assert!(c.get("poddojo").is_some());
        assert_eq!(c.launcher.unwrap().mac_bundle_id, "dev.bujosa.keiko");
    }

    fn app() -> App {
        Catalog::bundled().unwrap().apps[0].clone()
    }

    #[test]
    fn asset_name_fills_placeholders() {
        assert_eq!(app().asset_name("0.3.0", "arm64"), "PodDojo-v0.3.0-macos-arm64.zip");
    }

    #[test]
    fn rejects_unsafe_entries() {
        let mut a = app();
        a.repo = "../evil".into();
        assert!(a.validate().is_err());
        let mut a = app();
        a.mac_bundle_name = "../Evil".into();
        assert!(a.validate().is_err());
        let mut a = app();
        a.id = "Pod Dojo".into();
        assert!(a.validate().is_err());
        let mut a = app();
        a.asset = "x.dmg".into();
        assert!(a.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_ids() {
        let text =
            format!("{}\n{}", BUNDLED, BUNDLED.split("[[apps]]").nth(1).map(|s| format!("[[apps]]{s}")).unwrap());
        assert!(Catalog::parse(&text).unwrap_err().contains("duplicate"));
    }

    #[test]
    fn splits_dojo_names() {
        assert_eq!(app().name_parts(), ("Pod", "Dojo"));
        let mut a = app();
        a.name = "Dojo".into();
        assert_eq!(a.name_parts(), ("Dojo", ""));
    }
}
