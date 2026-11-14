//! Keiko's model: the catalog of dojos, GitHub releases, what state each app is in, and the
//! progress a dojo reports. Pure: no processes, no network, no file system.

pub mod catalog;
pub mod format;
pub mod progress;
pub mod release;
pub mod state;

pub use catalog::{App, Catalog, ProgressSpec};
pub use progress::Snapshot;
pub use release::{Asset, Latest, Release};
pub use state::State;

/// The architecture name used in release asset names (`arm64`, `x86_64`).
pub fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    }
}

/// Seconds since the Unix epoch (0 if the clock is before it).
pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
