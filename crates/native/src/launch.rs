//! Open apps, folders and web pages; read a dojo's progress.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use keiko_core::{ProgressSpec, Snapshot, progress};

use crate::run::run;

fn open_args(args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let o = run(Command::new("/usr/bin/open").args(args), None, Duration::from_secs(15), 4096)?;
    if o.ok() { Ok(()) } else { Err(format!("open: {}", o.reason())) }
}

/// Launch an installed app.
pub fn open_app(path: &Path) -> Result<(), String> {
    open_args(&[path.as_os_str()])
}

/// Show an app in Finder.
pub fn reveal(path: &Path) -> Result<(), String> {
    open_args(&["-R".as_ref(), path.as_os_str()])
}

/// Open an https page in the default browser.
pub fn open_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err(format!("refusing to open {url}"));
    }
    open_args(&[url.as_ref()])
}

/// Run the dojo's progress command from inside its bundle and read the result.
pub fn progress(app_path: &Path, spec: &ProgressSpec) -> Option<Snapshot> {
    let bin = app_path.join("Contents/MacOS").join(&spec.bin);
    if !bin.is_file() {
        return None;
    }
    let o = run(Command::new(bin).args(&spec.args), None, Duration::from_secs(10), 1 << 20).ok()?;
    if !o.ok() {
        return None;
    }
    progress::extract(&String::from_utf8_lossy(&o.stdout), spec)
}
