//! Где лаунчер хранит данные.
//!
//! ```text
//! <root>/
//! ├─ config.toml        настройки лаунчера
//! ├─ logs/              логи (по дням)
//! ├─ profiles/          игровые профили: у каждого свои mods/, config/, saves/
//! ├─ shared/            общее для всех профилей, качается один раз
//! │  ├─ assets/         звуки, текстуры, языки (по хешам)
//! │  ├─ libraries/      jar-библиотеки игры и загрузчиков
//! │  ├─ versions/       JSON и jar версий Minecraft
//! │  └─ runtimes/       Java нужных версий
//! └─ cache/             временное, можно удалять
//! ```
//!
//! Корень по умолчанию — стандартная папка данных ОС
//! (`%APPDATA%\Cuber\Cuber\data` на Windows, `~/.local/share/cuber` на Linux,
//! `~/Library/Application Support/net.Cuber.Cuber` на macOS).
//! Переменная окружения `CUBER_HOME` переопределяет корень — удобно для разработки.

use std::path::{Path, PathBuf};

use crate::error::{Error, IoContext, Result};

/// Имя переменной окружения, переопределяющей корень данных.
pub const HOME_ENV: &str = "CUBER_HOME";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    /// Корень из `CUBER_HOME` или стандартная папка данных ОС.
    pub fn discover() -> Result<Self> {
        if let Some(custom) = std::env::var_os(HOME_ENV).filter(|v| !v.is_empty()) {
            return Ok(Self::at(PathBuf::from(custom)));
        }
        let dirs =
            directories::ProjectDirs::from("net", "Cuber", "Cuber").ok_or(Error::NoHomeDir)?;
        Ok(Self::at(dirs.data_dir()))
    }

    /// Явно заданный корень (тесты, переносная версия).
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Создаёт все нужные папки, если их ещё нет.
    pub fn ensure(&self) -> Result<()> {
        for dir in [
            self.root.clone(),
            self.logs(),
            self.profiles(),
            self.assets(),
            self.libraries(),
            self.versions(),
            self.runtimes(),
            self.cache(),
        ] {
            std::fs::create_dir_all(&dir).at(&dir)?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }
    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }
    pub fn profiles(&self) -> PathBuf {
        self.root.join("profiles")
    }
    pub fn shared(&self) -> PathBuf {
        self.root.join("shared")
    }
    pub fn assets(&self) -> PathBuf {
        self.shared().join("assets")
    }
    pub fn libraries(&self) -> PathBuf {
        self.shared().join("libraries")
    }
    pub fn versions(&self) -> PathBuf {
        self.shared().join("versions")
    }
    pub fn runtimes(&self) -> PathBuf {
        self.shared().join("runtimes")
    }
    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_creates_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::at(tmp.path().join("cuber"));
        paths.ensure().unwrap();

        for dir in [
            paths.logs(),
            paths.profiles(),
            paths.assets(),
            paths.libraries(),
            paths.versions(),
            paths.runtimes(),
            paths.cache(),
        ] {
            assert!(dir.is_dir(), "нет папки {}", dir.display());
        }
        // Повторный вызов не должен падать.
        paths.ensure().unwrap();
    }
}
