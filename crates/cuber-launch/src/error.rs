use std::path::PathBuf;

/// Ошибки установки и запуска.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("нет связи с {url}: {source}")]
    Http {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("сервер {url} ответил кодом {status}")]
    Status { url: String, status: u16 },

    #[error("файл {path} повреждён: ожидался SHA-1 {expected}, получен {actual}")]
    Hash {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("файл {path} неполный: ожидалось {expected} байт, получено {actual}")]
    Size {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },

    #[error("ошибка ввода-вывода ({path}): {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("не удалось разобрать {what}: {source}")]
    Json {
        what: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("неизвестная версия Minecraft: {0}")]
    UnknownVersion(String),

    #[error(
        "для этой системы нет Java {component} от Mojang — укажите свою Java в настройках профиля"
    )]
    NoJavaRuntime { component: String },

    #[error("не удалось распаковать {path}: {message}")]
    Zip { path: PathBuf, message: String },

    #[error("профиль: {0}")]
    Profile(String),

    #[error("не удалось запустить игру: {0}")]
    Launch(String),

    #[error("загрузка отменена")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, Error>;

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

impl Error {
    /// Стоит ли повторить попытку (временный сбой сети, обрыв, 5xx).
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Http { .. } => true,
            Error::Status { status, .. } => *status >= 500 || *status == 429 || *status == 408,
            Error::Hash { .. } | Error::Size { .. } => true,
            _ => false,
        }
    }
}
