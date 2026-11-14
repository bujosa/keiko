//! GitHub's REST API through `curl`.
//!
//! Public repositories need no account. For private ones Keiko uses a token from `KEIKO_GITHUB_TOKEN`,
//! `GITHUB_TOKEN` or `GH_TOKEN`, else the GitHub CLI's (`gh auth token`). The token is only ever
//! sent to `https://api.github.com`, passed to curl on stdin (never on the command line), and curl
//! drops it when GitHub redirects a download to its file storage.

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use keiko_core::{App, Latest, release};

use crate::run::run;

const API: &str = "https://api.github.com/";
const MAX_JSON: usize = 4 * 1024 * 1024;
/// Largest app download Keiko accepts.
pub const MAX_DOWNLOAD: u64 = 1024 * 1024 * 1024;

fn token() -> Option<&'static str> {
    static TOKEN: OnceLock<Option<String>> = OnceLock::new();
    TOKEN
        .get_or_init(|| {
            for var in ["KEIKO_GITHUB_TOKEN", "GITHUB_TOKEN", "GH_TOKEN"] {
                if let Ok(v) = std::env::var(var)
                    && valid_token(&v)
                {
                    return Some(v.trim().to_string());
                }
            }
            if std::env::var_os("KEIKO_NO_GH").is_some() {
                return None;
            }
            let o = run(Command::new(gh_path()?).args(["auth", "token"]), None, Duration::from_secs(5), 4096).ok()?;
            let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
            (o.ok() && valid_token(&t)).then_some(t)
        })
        .as_deref()
}

/// Apps opened from Finder get a minimal PATH, so look for `gh` where installers put it.
fn gh_path() -> Option<String> {
    ["/opt/homebrew/bin/gh", "/usr/local/bin/gh", "/usr/bin/gh"]
        .into_iter()
        .find(|p| Path::new(p).exists())
        .map(String::from)
}

fn valid_token(t: &str) -> bool {
    let t = t.trim();
    !t.is_empty() && t.len() < 512 && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether requests are authenticated (shown in the footer).
pub fn signed_in() -> bool {
    token().is_some()
}

fn curl(url: &str, accept: &str) -> Result<(Command, Option<Vec<u8>>), String> {
    if !url.starts_with(API) {
        return Err(format!("refusing a non-GitHub API address: {url}"));
    }
    let mut cmd = Command::new("/usr/bin/curl");
    cmd.args([
        "-sS",
        "-L",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--connect-timeout",
        "15",
        "-A",
        concat!("Keiko/", env!("CARGO_PKG_VERSION")),
        "-H",
        &format!("Accept: {accept}"),
        "-H",
        "X-GitHub-Api-Version: 2022-11-28",
    ]);
    let stdin = token().map(|t| {
        cmd.args(["-H", "@-"]);
        format!("Authorization: Bearer {t}\n").into_bytes()
    });
    Ok((cmd, stdin))
}

/// GET an API document: (HTTP status, body).
fn get(url: &str, accept: &str, max: usize) -> Result<(u16, Vec<u8>), String> {
    let (mut cmd, stdin) = curl(url, accept)?;
    cmd.args(["--max-time", "30", "-w", "\n%{http_code}", url]);
    let o = run(&mut cmd, stdin.as_deref(), Duration::from_secs(40), max + 16)?;
    if !o.ok() {
        return Err(format!("GitHub: {}", o.reason()));
    }
    let mut body = o.stdout;
    let cut = body.iter().rposition(|b| *b == b'\n').ok_or("GitHub: empty response")?;
    let status = String::from_utf8_lossy(&body[cut + 1..]).trim().parse::<u16>().map_err(|e| e.to_string())?;
    body.truncate(cut);
    Ok((status, body))
}

/// The latest release of `app`'s repository for this Mac.
pub fn latest(app: &App) -> Result<Option<Latest>, String> {
    let url = format!("{API}repos/{}/releases/latest", app.repo);
    let (status, body) = get(&url, "application/vnd.github+json", MAX_JSON)?;
    match status {
        200 => release::latest(app, &String::from_utf8_lossy(&body), keiko_core::arch()),
        404 if token().is_some() => Ok(None),
        404 => Err("no public release (a private repository needs `gh auth login`)".into()),
        401 => Err("GitHub refused the token; run `gh auth login` again".into()),
        403 | 429 => Err("GitHub's rate limit was reached; try again in a few minutes".into()),
        s => Err(format!("GitHub answered HTTP {s}")),
    }
}

/// A small release asset as text (the checksums file).
pub fn text(asset_url: &str) -> Result<String, String> {
    let (status, body) = get(asset_url, "application/octet-stream", 1024 * 1024)?;
    if status != 200 {
        return Err(format!("GitHub answered HTTP {status} for the checksums"));
    }
    String::from_utf8(body).map_err(|_| "the checksums file is not text".to_string())
}

/// Download a release asset to `dest`, calling `progress` with the bytes written so far.
pub fn download(asset_url: &str, dest: &Path, size: u64, progress: &dyn Fn(u64)) -> Result<(), String> {
    if size > MAX_DOWNLOAD {
        return Err("the download is larger than 1 GB".into());
    }
    let (mut cmd, stdin) = curl(asset_url, "application/octet-stream")?;
    cmd.args(["--fail", "--max-filesize", &MAX_DOWNLOAD.to_string(), "--speed-limit", "1024", "--speed-time", "60"]);
    cmd.arg("-o").arg(dest).arg(asset_url);
    let mut child = cmd
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    if let (Some(data), Some(mut pipe)) = (stdin, child.stdin.take()) {
        use std::io::Write;
        pipe.write_all(&data).map_err(|e| format!("curl: {e}"))?;
    }
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() > Duration::from_secs(3600) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("the download took longer than an hour".into());
            }
            Ok(None) => {
                progress(std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0));
                std::thread::sleep(Duration::from_millis(120));
            }
            Err(e) => return Err(format!("curl: {e}")),
        }
    };
    if !status.success() {
        let mut err = String::new();
        if let Some(mut e) = child.stderr.take() {
            use std::io::Read;
            let _ = e.by_ref().take(4096).read_to_string(&mut err);
        }
        let line = err.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("download failed");
        return Err(format!("download: {line}"));
    }
    progress(std::fs::metadata(dest).map(|m| m.len()).unwrap_or(size));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_talks_to_the_api() {
        assert!(curl("https://evil.example/x", "a").is_err());
        assert!(curl("http://api.github.com/x", "a").is_err());
        assert!(curl("https://api.github.com.evil.example/x", "a").is_err());
    }

    #[test]
    fn token_shape() {
        assert!(valid_token("gho_abc123"));
        assert!(!valid_token("abc\nHost: evil"));
        assert!(!valid_token(""));
    }
}
