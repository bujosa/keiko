//! Install, update and uninstall apps.
//!
//! An install only lands if every check passes: the zip's SHA-256 matches the release's checksums
//! file, the bundle inside has the catalog's identifier and a valid code signature. The previous
//! version is kept aside until the new one is in place and restored if anything fails.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use keiko_core::{App, Latest, release};
use sha2::{Digest, Sha256};

use crate::detect::{self, Installed};
use crate::run::run;
use crate::{Paths, github};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Checksums,
    Downloading { done: u64, total: u64 },
    Verifying,
    Installing,
}

/// SHA-256 of a file, lowercase hex.
pub fn sha256(path: &Path) -> Result<String, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn tool(program: &str, args: &[&std::ffi::OsStr], what: &str) -> Result<(), String> {
    let o = run(Command::new(program).args(args), None, Duration::from_secs(300), 1 << 20)?;
    if o.ok() { Ok(()) } else { Err(format!("{what}: {}", o.reason())) }
}

/// Download, verify and install `latest` of `app`, replacing `current` if there is one.
pub fn install(
    paths: &Paths,
    app: &App,
    latest: &Latest,
    current: Option<&Installed>,
    on: &dyn Fn(Phase),
) -> Result<Installed, String> {
    let asset = latest.asset.as_ref().ok_or("this release has no build for this Mac")?;
    let sums = latest.checksums.as_ref().ok_or_else(|| format!("this release has no {}", app.checksums))?;

    on(Phase::Checksums);
    let expected = release::checksum_for(&github::text(&sums.url)?, &asset.name)
        .ok_or_else(|| format!("{} does not list {}", app.checksums, asset.name))?;

    let work = paths.cache.join(format!("{}-{}", app.id, latest.version));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    let result = install_from(paths, app, latest, current, on, asset, &expected, &work);
    let _ = std::fs::remove_dir_all(&work);
    result
}

#[allow(clippy::too_many_arguments)]
fn install_from(
    paths: &Paths,
    app: &App,
    latest: &Latest,
    current: Option<&Installed>,
    on: &dyn Fn(Phase),
    asset: &keiko_core::Asset,
    expected: &str,
    work: &Path,
) -> Result<Installed, String> {
    let zip = work.join(&asset.name);
    on(Phase::Downloading { done: 0, total: asset.size });
    github::download(&asset.url, &zip, asset.size, &|done| on(Phase::Downloading { done, total: asset.size }))?;

    on(Phase::Verifying);
    let actual = sha256(&zip)?;
    if actual != expected {
        return Err(format!("checksum mismatch for {}: the download was not installed", asset.name));
    }
    on(Phase::Installing);
    let unpacked = work.join("unpacked");
    tool("/usr/bin/ditto", &["-x".as_ref(), "-k".as_ref(), zip.as_os_str(), unpacked.as_os_str()], "unzip")?;
    let bundle = format!("{}.app", app.mac_bundle_name);
    let fresh = unpacked.join(&bundle);
    let meta = std::fs::symlink_metadata(&fresh).map_err(|_| format!("the download does not contain {bundle}"))?;
    if !meta.is_dir() {
        return Err(format!("{bundle} in the download is not an app bundle"));
    }
    check_bundle(&fresh, app)?;
    let version = detect::plist_value(&fresh, "CFBundleShortVersionString").unwrap_or_else(|| latest.version.clone());

    let dir = current.and_then(|c| c.path.parent().map(Path::to_path_buf)).unwrap_or_else(|| paths.apps.clone());
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let dest = dir.join(&bundle);
    place(&fresh, &dest, app)?;
    Ok(Installed { path: dest, version })
}

/// The bundle carries the catalog's identifier and a valid signature.
pub fn check_bundle(bundle: &Path, app: &App) -> Result<(), String> {
    let id = detect::plist_value(bundle, "CFBundleIdentifier").unwrap_or_default();
    if id != app.mac_bundle_id {
        return Err(format!("the bundle identifier is {id:?}, expected {}", app.mac_bundle_id));
    }
    tool(
        "/usr/bin/codesign",
        &["--verify".as_ref(), "--deep".as_ref(), "--strict".as_ref(), bundle.as_os_str()],
        "code signature",
    )
}

/// Move `fresh` to `dest`, keeping the previous app aside until the move succeeded.
fn place(fresh: &Path, dest: &Path, app: &App) -> Result<(), String> {
    let mut aside: Option<PathBuf> = None;
    if std::fs::symlink_metadata(dest).is_ok() {
        // Only ever replace the same app.
        if detect::plist_value(dest, "CFBundleIdentifier").as_deref() != Some(app.mac_bundle_id.as_str()) {
            return Err(format!("{} is another app; Keiko leaves it alone", dest.display()));
        }
        let parked = dest.with_file_name(format!(".{}.keiko-previous-{}", app.mac_bundle_name, keiko_core::now()));
        std::fs::rename(dest, &parked).map_err(|e| format!("cannot replace {}: {e}", dest.display()))?;
        aside = Some(parked);
    }
    let moved = std::fs::rename(fresh, dest).or_else(|_| {
        // Different volumes: copy with ditto, which keeps signatures and extended attributes.
        tool("/usr/bin/ditto", &[fresh.as_os_str(), dest.as_os_str()], "copy")
    });
    match (moved, aside) {
        (Ok(()), Some(parked)) => {
            let _ = std::fs::remove_dir_all(parked);
            Ok(())
        }
        (Ok(()), None) => Ok(()),
        (Err(e), parked) => {
            let _ = std::fs::remove_dir_all(dest);
            if let Some(parked) = parked {
                let _ = std::fs::rename(parked, dest);
            }
            Err(e)
        }
    }
}

/// Move an installed app to the Trash (it can be restored from there).
pub fn uninstall(paths: &Paths, app: &App, installed: &Installed) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(&installed.path).map_err(|e| format!("cannot read the app: {e}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Keiko only moves app directories to the Trash, not links or files".into());
    }
    if detect::plist_value(&installed.path, "CFBundleIdentifier").as_deref() != Some(app.mac_bundle_id.as_str()) {
        return Err(format!("{} is not {}", installed.path.display(), app.name));
    }
    std::fs::create_dir_all(&paths.trash).map_err(|e| format!("cannot open the Trash; app kept: {e}"))?;
    for index in 0..1000 {
        let name = if index == 0 {
            format!("{}.app", app.mac_bundle_name)
        } else {
            format!("{} ({index}).app", app.mac_bundle_name)
        };
        let target = paths.trash.join(name);
        // Claim an empty destination atomically, including when a dangling link exists.
        match std::fs::create_dir(&target) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot reserve a place in the Trash; app kept: {e}")),
        }
        // A rename preserves the complete bundle. Never copy over another trashed app or
        // delete the source after a cross-volume copy: failures leave the installed app intact.
        return match std::fs::rename(&installed.path, &target) {
            Ok(()) => Ok(target),
            Err(e) => {
                let _ = std::fs::remove_dir(&target); // Only our empty reservation.
                Err(format!("cannot move {} to the Trash; app kept: {e}. You can use Finder instead", app.name))
            }
        };
    }
    Err("the Trash already has too many copies of this app; app kept".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("keiko-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn fake_app(dir: &Path, id: &str, version: &str) -> PathBuf {
        let app = dir.join("PodDojo.app");
        std::fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
        std::fs::write(
            app.join("Contents/Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{id}</string>
<key>CFBundleShortVersionString</key><string>{version}</string>
</dict></plist>"#
            ),
        )
        .unwrap();
        app
    }

    fn app() -> App {
        keiko_core::Catalog::bundled().unwrap().apps[0].clone()
    }

    #[test]
    fn hashes_files() {
        let d = tmp("hash");
        let f = d.join("x");
        std::fs::write(&f, b"abc").unwrap();
        assert_eq!(sha256(&f).unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn detects_and_replaces_only_the_same_app() {
        let d = tmp("place");
        let paths = Paths::for_tests(&d);
        fake_app(&paths.apps, "dev.bujosa.poddojo", "0.2.1");
        let found = detect::installed(&paths, &app()).unwrap();
        assert_eq!(found.version, "0.2.1");

        let fresh = fake_app(&d.join("new"), "dev.bujosa.poddojo", "0.3.0");
        place(&fresh, &found.path, &app()).unwrap();
        assert_eq!(detect::installed(&paths, &app()).unwrap().version, "0.3.0");
        // Nothing parked is left behind.
        let left: Vec<_> = std::fs::read_dir(&paths.apps).unwrap().collect();
        assert_eq!(left.len(), 1);

        // Another app with the same name is never replaced.
        let other = tmp("other");
        let theirs = fake_app(&other, "com.example.other", "9.9.9");
        let fresh = fake_app(&d.join("new2"), "dev.bujosa.poddojo", "0.4.0");
        assert!(place(&fresh, &theirs, &app()).unwrap_err().contains("another app"));
        assert!(detect::plist_value(&theirs, "CFBundleIdentifier").unwrap() == "com.example.other");
    }

    #[test]
    fn ignores_apps_with_another_identifier() {
        let d = tmp("ident");
        let paths = Paths::for_tests(&d);
        fake_app(&paths.apps, "com.example.other", "1.0.0");
        assert!(detect::installed(&paths, &app()).is_none());
    }

    #[test]
    fn uninstall_can_be_restored_without_losing_learner_data() {
        let d = tmp("uninstall-restore");
        let paths = Paths::for_tests(&d);
        fake_app(&paths.apps, "dev.bujosa.poddojo", "0.3.2");
        std::fs::create_dir_all(&paths.data).unwrap();
        let progress = paths.data.join("progress.json");
        std::fs::write(&progress, b"learner progress stays here").unwrap();
        let installed = detect::installed(&paths, &app()).unwrap();
        std::fs::write(installed.path.join("Contents/MacOS/payload"), b"complete bundle").unwrap();

        let trashed = uninstall(&paths, &app(), &installed).unwrap();
        assert_eq!(trashed, paths.trash.join("PodDojo.app"));
        assert!(!installed.path.exists());
        assert!(detect::installed(&paths, &app()).is_none());
        assert_eq!(std::fs::read(trashed.join("Contents/MacOS/payload")).unwrap(), b"complete bundle");
        assert_eq!(std::fs::read(&progress).unwrap(), b"learner progress stays here");

        std::fs::rename(&trashed, &installed.path).unwrap();
        assert_eq!(detect::installed(&paths, &app()), Some(installed));
        assert_eq!(std::fs::read(&progress).unwrap(), b"learner progress stays here");
    }

    #[test]
    fn uninstall_never_overwrites_trashed_bundles_files_or_links() {
        let d = tmp("uninstall-collisions");
        let paths = Paths::for_tests(&d);
        fake_app(&paths.apps, "dev.bujosa.poddojo", "0.3.2");
        let old = fake_app(&paths.trash, "com.example.older", "0.1.0");
        let occupied = paths.trash.join("PodDojo (1).app");
        std::fs::write(&occupied, b"do not replace").unwrap();
        let link = paths.trash.join("PodDojo (2).app");
        std::os::unix::fs::symlink(d.join("missing"), &link).unwrap();

        let installed = detect::installed(&paths, &app()).unwrap();
        let trashed = uninstall(&paths, &app(), &installed).unwrap();
        assert_eq!(trashed, paths.trash.join("PodDojo (3).app"));
        assert_eq!(detect::plist_value(&old, "CFBundleIdentifier").as_deref(), Some("com.example.older"));
        assert_eq!(std::fs::read(occupied).unwrap(), b"do not replace");
        assert!(std::fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    }

    #[test]
    fn uninstall_refuses_other_identifiers_and_source_links() {
        let d = tmp("uninstall-identity");
        let paths = Paths::for_tests(&d);
        let other = fake_app(&paths.apps, "com.example.other", "9.9.9");
        let installed = Installed { path: other.clone(), version: "9.9.9".into() };
        assert!(uninstall(&paths, &app(), &installed).is_err());
        assert!(other.exists());
        assert!(!paths.trash.exists());

        let original = fake_app(&d.join("original"), "dev.bujosa.poddojo", "0.3.2");
        let linked = d.join("linked.app");
        std::os::unix::fs::symlink(&original, &linked).unwrap();
        let installed = Installed { path: linked.clone(), version: "0.3.2".into() };
        assert!(uninstall(&paths, &app(), &installed).is_err());
        assert!(original.exists());
        assert!(linked.exists());
        assert!(!paths.trash.exists());
    }

    #[test]
    fn unavailable_trash_and_failed_moves_keep_the_installed_bundle() {
        let d = tmp("uninstall-failure");
        let mut paths = Paths::for_tests(&d);
        fake_app(&paths.apps, "dev.bujosa.poddojo", "0.3.2");
        let installed = detect::installed(&paths, &app()).unwrap();
        std::fs::write(&paths.trash, b"not a directory").unwrap();
        assert!(uninstall(&paths, &app(), &installed).unwrap_err().contains("app kept"));
        assert_eq!(detect::installed(&paths, &app()), Some(installed.clone()));

        // Moving a directory into itself fails on the filesystem. The reservation is removed,
        // while the complete original bundle remains available to launch or retry.
        paths.trash = installed.path.join("cannot-move-inside-itself");
        assert!(uninstall(&paths, &app(), &installed).unwrap_err().contains("app kept"));
        assert!(!paths.trash.join("PodDojo.app").exists());
        assert_eq!(detect::installed(&paths, &app()), Some(installed));
    }
}
