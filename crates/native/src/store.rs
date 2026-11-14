//! Settings and the release cache, as JSON files in the data folder (written atomically).

use std::collections::BTreeMap;
use std::path::Path;

use keiko_core::Latest;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum View {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Sort {
    #[default]
    Name,
    Updated,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Auto,
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub view: View,
    pub sort: Sort,
    pub theme: Theme,
}

/// The last known release of each app and when it was checked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Feed {
    pub checked_at: u64,
    pub latest: BTreeMap<String, Latest>,
}

fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    std::fs::read(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn save<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent folder")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

impl Settings {
    pub fn load(p: &Paths) -> Settings {
        load(&p.data.join("settings.json"))
    }
    pub fn save(&self, p: &Paths) -> Result<(), String> {
        save(&p.data.join("settings.json"), self)
    }
}

impl Feed {
    pub fn load(p: &Paths) -> Feed {
        load(&p.data.join("feed.json"))
    }
    pub fn save(&self, p: &Paths) -> Result<(), String> {
        save(&p.data.join("feed.json"), self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_survives_garbage() {
        let d = std::env::temp_dir().join(format!("keiko-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let p = Paths::for_tests(&d);
        assert_eq!(Settings::load(&p), Settings::default());
        let s = Settings { view: View::List, sort: Sort::Status, theme: Theme::Light };
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p), s);
        std::fs::write(p.data.join("feed.json"), b"{not json").unwrap();
        assert_eq!(Feed::load(&p), Feed::default());
    }
}
