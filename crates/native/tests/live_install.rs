//! Installs each dojo’s latest public release into a temporary folder. Needs the network:
//! `KEIKO_NO_GH=1 cargo test -p keiko-native -- --ignored`.

use keiko_core::Catalog;
use keiko_native::{Paths, detect, github, install};

#[test]
#[ignore = "network"]
fn installs_and_updates_poddojo() -> Result<(), String> {
    installs_updates_and_preserves_on_bad_checksum("poddojo")
}

#[test]
#[ignore = "network"]
fn installs_and_updates_typedojo() -> Result<(), String> {
    installs_updates_and_preserves_on_bad_checksum("typedojo")
}

fn installs_updates_and_preserves_on_bad_checksum(id: &str) -> Result<(), String> {
    let root = std::env::temp_dir().join(format!("keiko-live-{id}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::for_tests(&root);
    let app = Catalog::bundled()?.get(id).ok_or_else(|| format!("missing catalog entry: {id}"))?.clone();
    let latest = github::latest(&app)?.ok_or_else(|| format!("no release for {id}"))?;
    let verified = std::cell::Cell::new(false);
    let installed = install::install(&paths, &app, &latest, None, &|p| {
        if matches!(p, install::Phase::Verifying) {
            verified.set(true);
        }
    })?;
    assert_eq!(installed.version, latest.version);
    assert_eq!(detect::installed(&paths, &app), Some(installed.clone()));
    assert!(verified.get());
    // Installing again replaces the same bundle in place.
    let again = install::install(&paths, &app, &latest, Some(&installed), &|_| {})?;
    assert_eq!(again.path, installed.path);
    // A tampered checksum is refused and the installed copy stays.
    let mut bad = latest.clone();
    bad.asset.as_mut().ok_or("missing release asset")?.name = format!("{id}-not-listed.zip");
    assert!(install::install(&paths, &app, &bad, Some(&again), &|_| {}).is_err());
    assert_eq!(detect::installed(&paths, &app), Some(again));
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
#[ignore = "network"]
fn keiko_updates_itself() {
    let root = std::env::temp_dir().join(format!("keiko-self-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::for_tests(&root);
    let launcher = Catalog::bundled().unwrap().launcher.unwrap();
    // An older Keiko where the running bundle would be.
    let old = paths.apps.join("Keiko.app");
    std::fs::create_dir_all(old.join("Contents/MacOS")).unwrap();
    std::fs::write(
        old.join("Contents/Info.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>dev.bujosa.keiko</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string></dict></plist>"#,
    )
    .unwrap();
    let current = keiko_native::Installed { path: old.clone(), version: "0.1.0".into() };
    let latest = github::latest(&launcher).unwrap().expect("a Keiko release");
    let updated = install::install(&paths, &launcher, &latest, Some(&current), &|_| {}).unwrap();
    assert_eq!(updated.path, old);
    assert_eq!(updated.version, latest.version);
    assert!(old.join("Contents/MacOS/keiko").is_file());
    let _ = std::fs::remove_dir_all(&root);
}
