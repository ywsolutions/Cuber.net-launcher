//! Прогресс подготовки к запуску — для кнопки «Играть».

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Получение описания версии.
    Resolving,
    /// Проверка Java.
    Java,
    /// Проверка уже скачанных файлов.
    Checking,
    /// Загрузка недостающих файлов.
    Downloading,
    /// Распаковка нативных библиотек и т. п.
    Unpacking,
    /// Запуск процесса игры.
    Launching,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Progress {
    pub stage: Stage,
    /// Скачано байт (для Downloading).
    pub done: u64,
    /// Всего байт (для Downloading).
    pub total: u64,
}

impl Progress {
    pub fn stage(stage: Stage) -> Self {
        Self {
            stage,
            done: 0,
            total: 0,
        }
    }

    pub fn download(done: u64, total: u64) -> Self {
        Self {
            stage: Stage::Downloading,
            done,
            total,
        }
    }

    /// Общая доля готовности 0..1 (загрузка занимает основную часть шкалы).
    pub fn fraction(&self) -> f64 {
        match self.stage {
            Stage::Resolving => 0.0,
            Stage::Java => 0.02,
            Stage::Checking => 0.04,
            Stage::Downloading if self.total > 0 => {
                0.05 + 0.9 * (self.done as f64 / self.total as f64)
            }
            Stage::Downloading => 0.05,
            Stage::Unpacking => 0.96,
            Stage::Launching => 0.99,
        }
    }
}
