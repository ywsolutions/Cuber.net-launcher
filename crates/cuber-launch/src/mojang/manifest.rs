//! Список всех версий: version_manifest_v2.json.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    /// release, snapshot, old_beta, old_alpha.
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub release_time: Option<String>,
}

impl VersionManifest {
    pub fn find(&self, id: &str) -> Option<&VersionEntry> {
        self.versions.iter().find(|v| v.id == id)
    }

    /// Релизы, от новых к старым (порядок как в манифесте).
    pub fn releases(&self) -> impl Iterator<Item = &VersionEntry> {
        self.versions.iter().filter(|v| v.kind == "release")
    }
}
