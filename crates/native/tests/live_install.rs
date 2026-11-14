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
