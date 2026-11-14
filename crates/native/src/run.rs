//! Run a command with a timeout and bounded output.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Output {
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

impl Output {
    pub fn ok(&self) -> bool {
        self.status == Some(0)
    }

    /// The first line of stderr, for error messages.
    pub fn reason(&self) -> String {
        let line = self.stderr.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
        if line.is_empty() { format!("exit status {:?}", self.status) } else { line.chars().take(200).collect() }
    }
}

/// Run `cmd`, feeding `stdin`, killing it after `timeout`. Output is capped at `max` bytes.
pub fn run(cmd: &mut Command, stdin: Option<&[u8]>, timeout: Duration, max: usize) -> Result<Output, String> {
    let program = cmd.get_program().to_string_lossy().to_string();
    let mut child = cmd
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program}: {e}"))?;
    if let (Some(data), Some(mut pipe)) = (stdin, child.stdin.take()) {
        pipe.write_all(data).map_err(|e| format!("{program}: {e}"))?;
    }
    let mut out = child.stdout.take().ok_or("no stdout")?;
    let mut err = child.stderr.take().ok_or("no stderr")?;
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out.by_ref().take(max as u64 + 1).read_to_end(&mut buf);
        // Drain the rest so the child never blocks on a full pipe.
        let _ = std::io::copy(&mut out, &mut std::io::sink());
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err.by_ref().take(64 * 1024).read_to_end(&mut buf);
        let _ = std::io::copy(&mut err, &mut std::io::sink());
        String::from_utf8_lossy(&buf).to_string()
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s.code(),
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{program}: timed out after {}s", timeout.as_secs()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("{program}: {e}")),
        }
    };
    let stdout = reader.join().map_err(|_| "reader failed")?;
    let stderr = err_reader.join().map_err(|_| "reader failed")?;
    if stdout.len() > max {
        return Err(format!("{program}: output larger than {max} bytes"));
    }
    Ok(Output { status, stdout, stderr })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_with_stdin_and_limits() {
        let o = run(Command::new("cat").arg("-"), Some(b"hello"), Duration::from_secs(5), 100).unwrap();
        assert!(o.ok());
        assert_eq!(o.stdout, b"hello");
        assert!(run(Command::new("cat").arg("-"), Some(b"hello"), Duration::from_secs(5), 2).is_err());
        assert!(
            run(Command::new("sleep").arg("5"), None, Duration::from_millis(200), 10)
                .unwrap_err()
                .contains("timed out")
        );
    }
}
