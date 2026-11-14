//! Installs PodDojo's latest release for real into a temporary folder. Needs the network (and, while
//! PodDojo is private, `gh auth login`): `cargo test -p keiko-native -- --ignored`.

use keiko_core::Catalog;
use keiko_native::{Paths, detect, github, install};

#[test]
#[ignore = "network"]
fn installs_and_updates_poddojo() {
    let root = std::env::temp_dir().join(format!("keiko-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::for_tests(&root);
    let app = Catalog::bundled().unwrap().get("poddojo").unwrap().clone();
    let latest = github::latest(&app).unwrap().expect("a release");
    let phases = std::sync::Mutex::new(vec![]);
    let installed = install::install(&paths, &app, &latest, None, &|p| phases.lock().unwrap().push(p)).unwrap();
    assert_eq!(installed.version, latest.version);
    assert_eq!(detect::installed(&paths, &app).unwrap(), installed);
    assert!(phases.lock().unwrap().iter().any(|p| matches!(p, install::Phase::Verifying)));
    // Installing again replaces the same bundle in place.
    let again = install::install(&paths, &app, &latest, Some(&installed), &|_| {}).unwrap();
    assert_eq!(again.path, installed.path);
    // A tampered checksum is refused and the installed copy stays.
    let mut bad = latest.clone();
    bad.asset.as_mut().unwrap().name = "PodDojo-not-listed.zip".into();
    assert!(install::install(&paths, &app, &bad, Some(&again), &|_| {}).is_err());
    assert!(detect::installed(&paths, &app).is_some());
    let _ = std::fs::remove_dir_all(&root);
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
