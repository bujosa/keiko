//! Keiko's side effects on macOS. Like PodDojo it drives the system's own tools (`curl`, `ditto`,
//! `plutil`, `codesign`, `open`) instead of linking an HTTP or TLS stack: the binary stays small and
//! proxies, certificates and keychains behave exactly as in the user's terminal.

pub mod detect;
pub mod github;
pub mod install;
pub mod launch;
pub mod notify;
pub mod paths;
pub mod run;
pub mod store;

pub use detect::Installed;
pub use paths::Paths;
