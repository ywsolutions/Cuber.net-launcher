use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use cuber_core::{Config, Paths};
use cuber_launch::{Installer, Profile, ProfileStore};

/// Общее состояние приложения, доступное всем командам Tauri.
pub struct AppState {
    pub paths: Paths,
    pub config: Mutex<Config>,
    pub installer: Arc<Installer>,
    pub profiles: ProfileStore,
    /// Профили, игра которых сейчас запущена.
    pub running: Mutex<HashSet<String>>,
}

impl AppState {
    pub fn new(paths: Paths, config: Config) -> cuber_launch::Result<Self> {
        Ok(Self {
            installer: Arc::new(Installer::new(paths.clone())?),
            profiles: ProfileStore::new(paths.profiles()),
            paths,
            config: Mutex::new(config),
            running: Mutex::new(HashSet::new()),
        })
    }

    /// Установщик с учётом настроек профиля (своя Java).
    pub fn installer_for(&self, profile: &Profile) -> Installer {
        Installer {
            paths: self.installer.paths.clone(),
            endpoints: self.installer.endpoints.clone(),
            downloader: self.installer.downloader.clone(),
            env: self.installer.env.clone(),
            java_override: profile.java_path.clone(),
        }
    }
}
