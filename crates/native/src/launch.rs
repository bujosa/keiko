//! Open apps, folders and web pages.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

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

/// Open `app` again once this process has quit (after Keiko replaced its own bundle).
pub fn relaunch(app: &Path) -> Result<(), String> {
    Command::new("/bin/sh")
        .args(["-c", "sleep 1; /usr/bin/open \"$0\""])
        .arg(app)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("relaunch: {e}"))
}
