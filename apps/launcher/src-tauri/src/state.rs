use std::sync::Mutex;

use cuber_core::{Config, Paths};

/// Общее состояние приложения, доступное всем командам Tauri.
pub struct AppState {
    pub paths: Paths,
    pub config: Mutex<Config>,
}

impl AppState {
    pub fn new(paths: Paths, config: Config) -> Self {
        Self {
            paths,
            config: Mutex::new(config),
        }
    }
}
