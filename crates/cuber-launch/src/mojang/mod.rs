//! Форматы официальных метаданных Minecraft (piston-meta.mojang.com).

pub mod assets;
pub mod library;
pub mod manifest;
pub mod rules;
pub mod version;

/// Ссылка на файл с хешем и размером — общий кирпич форматов Mojang.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FileRef {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
