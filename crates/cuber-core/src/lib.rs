//! Общее ядро Cuber.net.
//!
//! Здесь живёт то, что нужно всем остальным модулям лаунчера:
//! где лежат данные ([`paths`]), настройки ([`config`]) и логи ([`logging`]).
//! Crate не зависит от Tauri, поэтому его можно тестировать без окна.

pub mod config;
pub mod error;
pub mod logging;
pub mod paths;

pub use config::{AfterLaunch, Config, Theme};
pub use error::{Error, Result};
pub use paths::Paths;

/// Версия лаунчера из Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
