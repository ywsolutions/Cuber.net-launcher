use std::path::PathBuf;

/// Ошибки ядра.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("не удалось определить папку данных пользователя")]
    NoHomeDir,

    #[error("ошибка ввода-вывода ({path}): {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("повреждён файл настроек {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("не удалось сохранить настройки: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("недопустимое значение настройки: {0}")]
    InvalidConfig(String),

    #[error("не удалось запустить логирование: {0}")]
    Logging(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Короткий помощник: привязать ошибку ввода-вывода к пути.
pub(crate) trait IoContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| Error::Io {
            path: path.into(),
            source,
        })
    }
}
