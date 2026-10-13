//! Игровые профили. Каждый профиль — отдельная папка:
//!
//! ```text
//! profiles/<id>/
//! ├─ profile.toml     описание: версия, загрузчик, память…
//! ├─ minecraft/       папка игры: saves, mods, resourcepacks, options.txt…
//! └─ logs/            вывод запусков
//! ```
//!
//! Профили не делят ничего изменяемого: правка сборки одного профиля
//! не затрагивает другие.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Error, IoContext, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl Loader {
    pub fn display_name(self) -> &'static str {
        match self {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Quilt => "Quilt",
            Loader::Forge => "Forge",
            Loader::NeoForge => "NeoForge",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(skip)]
    pub id: String,
    pub name: String,
    /// Версия Minecraft, например "1.20.1".
    pub minecraft: String,
    #[serde(default)]
    pub loader: Loader,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<String>,
    /// Память для этого профиля, МБ (иначе — из общих настроек).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jvm_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<PathBuf>,
    #[serde(default)]
    pub created: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<u64>,
}

impl Profile {
    pub fn new(name: &str, minecraft: &str) -> Self {
        Self {
            id: String::new(),
            name: name.trim().to_string(),
            minecraft: minecraft.to_string(),
            loader: Loader::Vanilla,
            loader_version: None,
            memory_mb: None,
            jvm_args: vec![],
            java_path: None,
            created: now(),
            last_played: None,
        }
    }

    /// Id версии для установщика (для загрузчиков модов появится на следующих этапах).
    pub fn version_id(&self) -> String {
        self.minecraft.clone()
    }
}

pub struct ProfileStore {
    root: PathBuf,
}

impl ProfileStore {
    pub fn new(profiles_dir: impl Into<PathBuf>) -> Self {
        Self {
            root: profiles_dir.into(),
        }
    }

    pub fn dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    pub fn game_dir(&self, id: &str) -> PathBuf {
        self.dir(id).join("minecraft")
    }

    pub fn logs_dir(&self, id: &str) -> PathBuf {
        self.dir(id).join("logs")
    }

    /// Все профили: сначала недавно запускавшиеся, потом по имени.
    pub fn list(&self) -> Result<Vec<Profile>> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(&self.root) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e).at(&self.root),
        };
        for entry in entries.flatten() {
            let id = entry.file_name().to_string_lossy().into_owned();
            match self.load(&id) {
                Ok(p) => out.push(p),
                Err(e) => tracing::warn!(%e, id, "пропускаю повреждённый профиль"),
            }
        }
        out.sort_by(|a, b| {
            b.last_played
                .unwrap_or(0)
                .cmp(&a.last_played.unwrap_or(0))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(out)
    }

    pub fn load(&self, id: &str) -> Result<Profile> {
        validate_id(id)?;
        let path = self.dir(id).join("profile.toml");
        let text = std::fs::read_to_string(&path).at(&path)?;
        let mut profile: Profile = toml::from_str(&text)
            .map_err(|e| Error::Profile(format!("{}: {e}", path.display())))?;
        profile.id = id.to_string();
        Ok(profile)
    }

    /// Создать профиль с уникальным id на основе имени.
    pub fn create(&self, mut profile: Profile) -> Result<Profile> {
        if profile.name.is_empty() {
            return Err(Error::Profile("у профиля должно быть имя".into()));
        }
        let base = slug(&profile.name);
        let mut id = base.clone();
        let mut n = 2;
        while self.dir(&id).exists() {
            id = format!("{base}-{n}");
            n += 1;
        }
        profile.id = id;
        std::fs::create_dir_all(self.game_dir(&profile.id)).at(self.game_dir(&profile.id))?;
        self.save(&profile)?;
        Ok(profile)
    }

    pub fn save(&self, profile: &Profile) -> Result<()> {
        validate_id(&profile.id)?;
        let dir = self.dir(&profile.id);
        std::fs::create_dir_all(&dir).at(&dir)?;
        let text = toml::to_string_pretty(profile).map_err(|e| Error::Profile(e.to_string()))?;
        let path = dir.join("profile.toml");
        let tmp = dir.join("profile.toml.tmp");
        std::fs::write(&tmp, text).at(&tmp)?;
        std::fs::rename(&tmp, &path).at(&path)
    }

    pub fn mark_played(&self, id: &str) -> Result<()> {
        let mut p = self.load(id)?;
        p.last_played = Some(now());
        self.save(&p)
    }
}

/// id — имя папки: латиница, цифры, дефис. Без «..» и разделителей пути.
fn validate_id(id: &str) -> Result<()> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(Error::Profile(format!("недопустимый id профиля: {id}")))
    }
}

/// «Моя сборка 1.20!» → "moya-sborka-1-20".
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        let mapped = match c {
            'a'..='z' | '0'..='9' => c.to_string(),
            'а' => "a".into(),
            'б' => "b".into(),
            'в' => "v".into(),
            'г' => "g".into(),
            'д' => "d".into(),
            'е' | 'ё' | 'э' => "e".into(),
            'ж' => "zh".into(),
            'з' => "z".into(),
            'и' | 'й' => "i".into(),
            'к' => "k".into(),
            'л' => "l".into(),
            'м' => "m".into(),
            'н' => "n".into(),
            'о' => "o".into(),
            'п' => "p".into(),
            'р' => "r".into(),
            'с' => "s".into(),
            'т' => "t".into(),
            'у' => "u".into(),
            'ф' => "f".into(),
            'х' => "h".into(),
            'ц' => "c".into(),
            'ч' => "ch".into(),
            'ш' | 'щ' => "sh".into(),
            'ы' => "y".into(),
            'ю' => "yu".into(),
            'я' => "ya".into(),
            'ъ' | 'ь' => String::new(),
            _ => "-".into(),
        };
        out.push_str(&mapped);
    }
    let mut slug = String::new();
    for part in out.split('-').filter(|p| !p.is_empty()) {
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.push_str(part);
    }
    slug.truncate(40);
    let slug = slug.trim_end_matches('-').to_string();
    if slug.is_empty() {
        "profile".into()
    } else {
        slug
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(slug("Моя сборка 1.20!"), "moya-sborka-1-20");
        assert_eq!(slug("Cuber Survival"), "cuber-survival");
        assert_eq!(slug("!!!"), "profile");
    }

    #[test]
    fn create_list_isolated_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(tmp.path());
        let a = store.create(Profile::new("Survival", "1.20.1")).unwrap();
        let b = store.create(Profile::new("Survival", "1.21")).unwrap();
        assert_eq!(a.id, "survival");
        assert_eq!(b.id, "survival-2");
        assert_ne!(store.game_dir(&a.id), store.game_dir(&b.id));
        assert!(store.game_dir(&a.id).is_dir());

        store.mark_played(&b.id).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list[0].id, "survival-2");
        assert_eq!(list[1].minecraft, "1.20.1");
    }

    #[test]
    fn rejects_path_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(tmp.path());
        assert!(store.load("../etc").is_err());
    }
}
