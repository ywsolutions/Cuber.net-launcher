//! Логирование: в консоль и в файл `logs/launcher.YYYY-MM-DD.log`.
//!
//! Уровень задаётся переменной `CUBER_LOG` (синтаксис `RUST_LOG`),
//! по умолчанию `info`. Хранятся последние 7 файлов логов.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use crate::error::{Error, Result};

pub const LOG_ENV: &str = "CUBER_LOG";
const MAX_LOG_FILES: usize = 7;

/// Запускает логирование. Возвращённый guard нужно держать живым
/// до конца работы программы — иначе последние строки не попадут в файл.
pub fn init(logs_dir: &Path) -> Result<WorkerGuard> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("launcher")
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(logs_dir)
        .map_err(|e| Error::Logging(e.to_string()))?;
    let (file_writer, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_env(LOG_ENV).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .with(fmt::layer().with_ansi(false).with_writer(file_writer))
        .try_init()
        .map_err(|e| Error::Logging(e.to_string()))?;

    Ok(guard)
}
