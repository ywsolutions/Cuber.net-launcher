//! Библиотеки из JSON версии: путь в Maven-раскладке, нативные библиотеки.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::rules::{allowed, Env, Rule};
use crate::http::Download;

const MOJANG_LIBRARIES: &str = "https://libraries.minecraft.net/";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Library {
    /// group:artifact:version[:classifier][@ext]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    /// Базовый адрес Maven-репозитория (так задают библиотеки Fabric/Quilt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    /// Старый формат нативных библиотек: ОС → классификатор.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LibraryDownloads {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Что нужно сделать с библиотекой на этой системе.
#[derive(Debug, Clone, Default)]
pub struct Resolved {
    /// Файл в classpath.
    pub classpath: Option<PathBuf>,
    /// Нативный jar, который нужно распаковать в папку natives, и что исключить.
    pub native: Option<(PathBuf, Vec<String>)>,
    pub downloads: Vec<Download>,
}

impl Library {
    /// group:artifact[:classifier] — чтобы находить одну и ту же библиотеку разных версий.
    pub fn identity(&self) -> String {
        let parts: Vec<&str> = self
            .name
            .split('@')
            .next()
            .unwrap_or("")
            .split(':')
            .collect();
        match parts.as_slice() {
            [g, a, _v, c, ..] => format!("{g}:{a}:{c}"),
            [g, a, ..] => format!("{g}:{a}"),
            _ => self.name.clone(),
        }
    }

    pub fn applies(&self, env: &Env) -> bool {
        self.rules.as_deref().map_or(true, |r| allowed(r, env))
    }

    /// Разобрать библиотеку для окружения: classpath, натив и что скачать.
    pub fn resolve(&self, env: &Env, libraries_dir: &Path) -> Resolved {
        let mut out = Resolved::default();
        if !self.applies(env) {
            return out;
        }

        // Основной артефакт.
        let has_old_natives = self.natives.is_some();
        let artifact = self.downloads.as_ref().and_then(|d| d.artifact.as_ref());
        let main = match artifact {
            Some(a) => Some(artifact_download(a, &self.name, None, libraries_dir)),
            // Нет блока downloads: Maven-раскладка по url (Fabric) или репозиторий Mojang.
            // У старых нативных библиотек основного артефакта может не быть вовсе.
            None if self.downloads.is_none() => {
                let rel = maven_path(&self.name, None);
                let base = self.url.as_deref().unwrap_or(MOJANG_LIBRARIES);
                let mut d = Download::new(join_url(base, &rel), libraries_dir.join(&rel));
                if let Some(sha1) = &self.sha1 {
                    d = d.sha1(sha1);
                }
                if let Some(size) = self.size {
                    d = d.size(size);
                }
                Some(d)
            }
            None => None,
        };
        if let Some(d) = main {
            // Новый формат: натив — обычная библиотека с классификатором natives-*.
            if classifier(&self.name).is_some_and(|c| c.starts_with("natives-")) {
                out.native = Some((d.path.clone(), vec!["META-INF/".into()]));
            }
            out.classpath = Some(d.path.clone());
            out.downloads.push(d);
        }

        // Старый формат: natives → классификатор → classifiers.
        if has_old_natives {
            if let Some(key) = self.natives.as_ref().and_then(|n| n.get(env.os)) {
                let bits = if env.is_64bit() { "64" } else { "32" };
                let key = key.replace("${arch}", bits);
                let classified = self
                    .downloads
                    .as_ref()
                    .and_then(|d| d.classifiers.as_ref())
                    .and_then(|c| c.get(&key));
                let d = match classified {
                    Some(a) => artifact_download(a, &self.name, Some(&key), libraries_dir),
                    None => {
                        let rel = maven_path(&self.name, Some(&key));
                        let base = self.url.as_deref().unwrap_or(MOJANG_LIBRARIES);
                        Download::new(join_url(base, &rel), libraries_dir.join(&rel))
                    }
                };
                let exclude = self
                    .extract
                    .as_ref()
                    .map(|e| e.exclude.clone())
                    .unwrap_or_default();
                out.native = Some((d.path.clone(), exclude));
                out.downloads.push(d);
            }
        }
        out
    }
}

fn artifact_download(a: &Artifact, name: &str, cls: Option<&str>, libs: &Path) -> Download {
    let rel = a.path.clone().unwrap_or_else(|| maven_path(name, cls));
    let url = match a.url.as_deref() {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => join_url(MOJANG_LIBRARIES, &rel),
    };
    let mut d = Download::new(url, libs.join(&rel));
    if let Some(sha1) = &a.sha1 {
        d = d.sha1(sha1);
    }
    if let Some(size) = a.size {
        d = d.size(size);
    }
    d
}

fn classifier(name: &str) -> Option<&str> {
    name.split('@').next()?.split(':').nth(3)
}

/// group:artifact:version[:classifier][@ext] → group/path/artifact/version/artifact-version[-classifier].ext
pub fn maven_path(name: &str, extra_classifier: Option<&str>) -> String {
    let (coords, ext) = match name.split_once('@') {
        Some((c, e)) => (c, e),
        None => (name, "jar"),
    };
    let parts: Vec<&str> = coords.split(':').collect();
    let (group, artifact, version) = (
        parts.first().copied().unwrap_or(""),
        parts.get(1).copied().unwrap_or(""),
        parts.get(2).copied().unwrap_or(""),
    );
    let cls = extra_classifier.or_else(|| parts.get(3).copied());
    let file = match cls {
        Some(c) => format!("{artifact}-{version}-{c}.{ext}"),
        None => format!("{artifact}-{version}.{ext}"),
    };
    format!("{}/{artifact}/{version}/{file}", group.replace('.', "/"))
}

fn join_url(base: &str, rel: &str) -> String {
    if base.ends_with('/') {
        format!("{base}{rel}")
    } else {
        format!("{base}/{rel}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(os: &'static str) -> Env {
        Env {
            os,
            arch: "x86_64",
            features: Default::default(),
        }
    }

    #[test]
    fn maven_paths() {
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.1", None),
            "org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1.jar"
        );
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.1:natives-windows", None),
            "org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-windows.jar"
        );
        assert_eq!(
            maven_path(
                "net.minecraftforge:forge:1.20.1-47.2.0@zip",
                Some("installer")
            ),
            "net/minecraftforge/forge/1.20.1-47.2.0/forge-1.20.1-47.2.0-installer.zip"
        );
    }

    #[test]
    fn fabric_style_library_uses_url_base() {
        let lib: Library = serde_json::from_str(
            r#"{"name":"net.fabricmc:intermediary:1.20.1","url":"https://maven.fabricmc.net/"}"#,
        )
        .unwrap();
        let r = lib.resolve(&env("linux"), Path::new("/libs"));
        assert_eq!(
            r.downloads[0].url,
            "https://maven.fabricmc.net/net/fabricmc/intermediary/1.20.1/intermediary-1.20.1.jar"
        );
        assert!(r.native.is_none());
    }

    #[test]
    fn old_style_natives_pick_classifier() {
        let lib: Library = serde_json::from_str(
            r#"{
            "name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.4-nightly-20150209",
            "downloads": {"classifiers": {
                "natives-linux": {"path":"a/linux.jar","sha1":"aa","size":1,"url":"https://x/linux.jar"},
                "natives-windows": {"path":"a/win.jar","sha1":"bb","size":2,"url":"https://x/win.jar"}
            }},
            "natives": {"linux":"natives-linux","windows":"natives-windows"},
            "extract": {"exclude":["META-INF/"]}
        }"#,
        )
        .unwrap();
        let r = lib.resolve(&env("windows"), Path::new("/libs"));
        assert!(r.classpath.is_none());
        assert_eq!(r.downloads.len(), 1);
        assert_eq!(r.downloads[0].url, "https://x/win.jar");
        let (path, exclude) = r.native.unwrap();
        assert!(path.ends_with("a/win.jar"));
        assert_eq!(exclude, ["META-INF/"]);
    }

    #[test]
    fn new_style_natives_are_classpath_and_extracted() {
        let lib: Library = serde_json::from_str(
            r#"{
            "name": "org.lwjgl:lwjgl:3.3.1:natives-windows",
            "downloads": {"artifact": {"path":"org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-windows.jar",
                "sha1":"cc","size":3,"url":"https://libraries.minecraft.net/x.jar"}},
            "rules": [{"action":"allow","os":{"name":"windows"}}]
        }"#,
        )
        .unwrap();
        assert!(lib
            .resolve(&env("linux"), Path::new("/l"))
            .downloads
            .is_empty());
        let r = lib.resolve(&env("windows"), Path::new("/l"));
        assert!(r.classpath.is_some());
        assert!(r.native.is_some());
    }

    #[test]
    fn identity_ignores_version() {
        let a: Library = serde_json::from_str(r#"{"name":"org.ow2.asm:asm:9.3"}"#).unwrap();
        let b: Library = serde_json::from_str(r#"{"name":"org.ow2.asm:asm:9.6"}"#).unwrap();
        assert_eq!(a.identity(), b.identity());
    }
}
