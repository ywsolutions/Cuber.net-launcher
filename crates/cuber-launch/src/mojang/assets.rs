//! Ассеты: звуки, языки, текстуры интерфейса. Хранятся один раз по хешу
//! и используются всеми профилями.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::http::Download;

pub const RESOURCES_BASE: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: BTreeMap<String, AssetObject>,
    /// 1.6–1.7.2: ассеты нужно разложить по именам в assets/virtual/<id>.
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    /// До 1.6: ассеты кладутся в папку resources внутри папки игры.
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    /// assets/objects/ab/abcdef…
    pub fn rel_path(&self) -> String {
        format!("{}/{}", &self.hash[..2], self.hash)
    }

    pub fn download(&self, base_url: &str, assets_dir: &Path) -> Download {
        let rel = self.rel_path();
        Download::new(
            format!("{}/{}", base_url.trim_end_matches('/'), rel),
            assets_dir.join("objects").join(&rel),
        )
        .sha1(&self.hash)
        .size(self.size)
    }
}
