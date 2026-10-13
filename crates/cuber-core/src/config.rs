//! Настройки лаунчера (`config.toml`).
//!
//! Принципы:
//! - отсутствующий файл = настройки по умолчанию;
//! - неизвестные и отсутствующие поля не ломают загрузку (новые версии
//!   лаунчера читают старые конфиги и наоборот);
//! - сохранение атомарное: пишем во временный файл и переименовываем,
//!   чтобы сбой посреди записи не оставил битый конфиг.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, IoContext, Result};

/// Минимум и максимум памяти для игры, МБ.
pub const MIN_MEMORY_MB: u32 = 512;
pub const MAX_MEMORY_MB: u32 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

/// Что делать с окном лаунчера после запуска игры.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfterLaunch {
    KeepOpen,
    /// По умолчанию: свернуть, чтобы не мешать и не тратить ресурсы.
    #[default]
    Minimize,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub after_launch: AfterLaunch,
    /// Память по умолчанию для новых профилей, МБ (-Xmx).
    pub default_memory_mb: u32,
    /// Последний введённый ник для офлайн-игры.
    pub last_nickname: Option<String>,
    /// Пройдено ли приветствие при первом запуске.
    pub onboarded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            after_launch: AfterLaunch::default(),
            default_memory_mb: 4096,
            last_nickname: None,
            onboarded: false,
        }
    }
}

impl Config {
    /// Загружает настройки; если файла нет — возвращает значения по умолчанию.
    pub fn load(path: &Path) -> Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e).at(path),
        };
        let config: Self = toml::from_str(&text).map_err(|source| Error::ConfigParse {
            path: path.to_path_buf(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Загружает настройки, а при повреждённом файле сохраняет его копию
    /// (`config.toml.broken`) и начинает с настроек по умолчанию.
    /// Лаунчер не должен отказываться запускаться из-за конфига.
    pub fn load_or_recover(path: &Path) -> Self {
        match Self::load(path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(%err, "настройки повреждены, использую значения по умолчанию");
                let backup = path.with_extension("toml.broken");
                if let Err(e) = std::fs::rename(path, &backup) {
                    tracing::warn!(%e, "не удалось сохранить копию повреждённых настроек");
                }
                Self::default()
            }
        }
    }

    /// Проверяет значения на допустимость.
    pub fn validate(&self) -> Result<()> {
        if !(MIN_MEMORY_MB..=MAX_MEMORY_MB).contains(&self.default_memory_mb) {
            return Err(Error::InvalidConfig(format!(
                "память должна быть от {MIN_MEMORY_MB} до {MAX_MEMORY_MB} МБ, получено {}",
                self.default_memory_mb
            )));
        }
        Ok(())
    }

    /// Атомарно сохраняет настройки.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let text = toml::to_string_pretty(self)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).at(dir)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text).at(&tmp)?;
        std::fs::rename(&tmp, path).at(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let config = Config::load(&tmp.path().join("config.toml")).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        let config = Config {
            theme: Theme::Dark,
            after_launch: AfterLaunch::Close,
            default_memory_mb: 6144,
            last_nickname: Some("SYGEN".into()),
            onboarded: true,
        };
        config.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), config);
        assert!(!path.with_extension("toml.tmp").exists());
    }

    #[test]
    fn partial_and_unknown_fields_are_tolerated() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "theme = \"light\"\nfuture_option = 42\n").unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.theme, Theme::Light);
        assert_eq!(
            config.default_memory_mb,
            Config::default().default_memory_mb
        );
    }

    #[test]
    fn invalid_memory_is_rejected() {
        let config = Config {
            default_memory_mb: 100,
            ..Config::default()
        };
        assert!(config.validate().is_err());
        let tmp = tempfile::tempdir().unwrap();
        assert!(config.save(&tmp.path().join("config.toml")).is_err());
    }

    #[test]
    fn broken_file_is_recovered() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "это не toml = = =").unwrap();
        let config = Config::load_or_recover(&path);
        assert_eq!(config, Config::default());
        assert!(path.with_extension("toml.broken").exists());
    }
}
