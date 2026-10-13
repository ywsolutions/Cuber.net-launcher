//! Java для игры — официальные сборки рантайма от Mojang (те же, что ставит
//! официальный лаунчер). Каждая версия Minecraft указывает нужный компонент:
//! jre-legacy (Java 8), java-runtime-gamma (17), java-runtime-delta (21) и т. д.
//!
//! Раскладка: runtimes/<компонент>/… и рядом <компонент>.sha1 — хеш манифеста
//! установленной сборки. Если он совпадает с актуальным — ничего не проверяем заново.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, IoContext, Result};
use crate::http::{Download, Downloader};
use crate::mojang::FileRef;

pub const RUNTIMES_INDEX: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// Платформа в терминах Mojang или None, если сборок для неё нет.
pub fn platform() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x64",
        ("windows", "x86") => "windows-x86",
        ("windows", "aarch64") => "windows-arm64",
        ("linux", "x86_64") => "linux",
        ("linux", "x86") => "linux-i386",
        ("macos", "x86_64") => "mac-os",
        ("macos", "aarch64") => "mac-os-arm64",
        _ => return None,
    })
}

type RuntimeIndex = HashMap<String, HashMap<String, Vec<RuntimeEntry>>>;

#[derive(Debug, Deserialize)]
struct RuntimeEntry {
    manifest: FileRef,
}

#[derive(Debug, Deserialize)]
struct RuntimeManifest {
    files: BTreeMap<String, RuntimeFile>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RuntimeFile {
    File {
        #[serde(default)]
        executable: bool,
        downloads: RuntimeDownloads,
    },
    Directory,
    Link {
        target: String,
    },
}

#[derive(Debug, Deserialize)]
struct RuntimeDownloads {
    raw: FileRef,
}

/// Подготовленная установка Java: что скачать и что сделать после.
pub struct JavaPlan {
    pub component: String,
    pub dir: PathBuf,
    pub java: PathBuf,
    pub downloads: Vec<Download>,
    executables: Vec<PathBuf>,
    links: Vec<(PathBuf, String)>,
    directories: Vec<PathBuf>,
    manifest_sha1: String,
    marker: PathBuf,
}

/// Путь к исполняемому java внутри папки рантайма.
pub fn java_executable(dir: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        dir.join("jre.bundle/Contents/Home/bin/java")
    } else if cfg!(windows) {
        dir.join("bin").join("java.exe")
    } else {
        dir.join("bin").join("java")
    }
}

/// Спланировать установку компонента. Если уже установлен — пустой план.
pub async fn plan(
    dl: &Downloader,
    index_url: &str,
    runtimes_dir: &Path,
    component: &str,
) -> Result<JavaPlan> {
    let dir = runtimes_dir.join(component);
    let java = java_executable(&dir);
    let marker = runtimes_dir.join(format!("{component}.sha1"));

    let Some(platform) = platform() else {
        return Err(Error::NoJavaRuntime {
            component: component.into(),
        });
    };

    let index: RuntimeIndex = match dl.get_json(index_url).await {
        Ok(index) => index,
        // Нет сети, но Java уже стоит — играем на установленной.
        Err(_) if java.exists() && marker.exists() => {
            return Ok(JavaPlan::installed(component, dir, java, marker))
        }
        Err(e) => return Err(e),
    };
    let entry = index
        .get(platform)
        .and_then(|p| p.get(component))
        .and_then(|v| v.first())
        .ok_or_else(|| Error::NoJavaRuntime {
            component: component.into(),
        })?;

    let installed = tokio::fs::read_to_string(&marker).await.unwrap_or_default();
    if installed.trim() == entry.manifest.sha1 && java.exists() {
        return Ok(JavaPlan::installed(component, dir, java, marker));
    }

    // Mojang обновил сборку: ставим начисто, чтобы не смешать файлы двух версий.
    // (Без отметки — значит прошлая установка оборвалась: докачиваем недостающее.)
    if !installed.trim().is_empty() && dir.exists() {
        tokio::fs::remove_dir_all(&dir).await.at(&dir)?;
    }

    let manifest: RuntimeManifest = dl.get_json(&entry.manifest.url).await?;
    let mut plan = JavaPlan::installed(component, dir.clone(), java, marker);
    plan.manifest_sha1 = entry.manifest.sha1.clone();
    for (rel, file) in manifest.files {
        let path = dir.join(&rel);
        match file {
            RuntimeFile::Directory => plan.directories.push(path),
            RuntimeFile::File {
                executable,
                downloads,
            } => {
                if executable {
                    plan.executables.push(path.clone());
                }
                plan.downloads.push(
                    Download::new(downloads.raw.url, path)
                        .sha1(downloads.raw.sha1)
                        .size(downloads.raw.size),
                );
            }
            RuntimeFile::Link { target } => plan.links.push((path, target)),
        }
    }
    Ok(plan)
}

impl JavaPlan {
    fn installed(component: &str, dir: PathBuf, java: PathBuf, marker: PathBuf) -> Self {
        Self {
            component: component.into(),
            dir,
            java,
            downloads: vec![],
            executables: vec![],
            links: vec![],
            directories: vec![],
            manifest_sha1: String::new(),
            marker,
        }
    }

    pub fn is_installed(&self) -> bool {
        self.manifest_sha1.is_empty()
    }

    /// Шаги после загрузки: права на запуск, ссылки, отметка об установке.
    pub async fn finish(&self) -> Result<()> {
        if self.is_installed() {
            return Ok(());
        }
        for dir in &self.directories {
            tokio::fs::create_dir_all(dir).await.at(dir)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in &self.executables {
                let perms = std::fs::Permissions::from_mode(0o755);
                tokio::fs::set_permissions(path, perms).await.at(path)?;
            }
            for (path, target) in &self.links {
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await.at(parent)?;
                }
                let _ = tokio::fs::remove_file(path).await;
                tokio::fs::symlink(target, path).await.at(path)?;
            }
        }
        if let Some(parent) = self.marker.parent() {
            tokio::fs::create_dir_all(parent).await.at(parent)?;
        }
        tokio::fs::write(&self.marker, &self.manifest_sha1)
            .await
            .at(&self.marker)
    }
}
