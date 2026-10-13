//! Установка версии Minecraft: всё, что нужно для запуска.
//!
//! Общие файлы (библиотеки, ассеты, client.jar, Java) лежат в shared/ и runtimes/
//! один раз для всех профилей: они неизменяемы и проверяются по хешу, поэтому
//! переиспользовать их безопасно. Всё, что игра меняет (миры, моды, настройки),
//! живёт в папке профиля.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use cuber_core::Paths;

use crate::error::{Error, IoContext, Result};
use crate::http::{Download, Downloader, OnBytes};
use crate::java;
use crate::mojang::assets::{AssetIndex, RESOURCES_BASE};
use crate::mojang::manifest::VersionManifest;
use crate::mojang::rules::Env;
use crate::mojang::version::VersionJson;
use crate::progress::{Progress, Stage};

pub const VERSION_MANIFEST: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Адреса официальных сервисов (в тестах подменяются на локальный сервер).
#[derive(Debug, Clone)]
pub struct Endpoints {
    pub version_manifest: String,
    pub java_runtimes: String,
    pub resources: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            version_manifest: VERSION_MANIFEST.into(),
            java_runtimes: java::RUNTIMES_INDEX.into(),
            resources: RESOURCES_BASE.into(),
        }
    }
}

/// Готовая к запуску версия.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub version: VersionJson,
    pub java: PathBuf,
    pub java_major: u32,
    pub client_jar: PathBuf,
    pub classpath: Vec<PathBuf>,
    pub natives_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub assets_root: PathBuf,
    pub asset_index_id: String,
    /// Папка для ${game_assets}: виртуальные ассеты старых версий или assets_root.
    pub game_assets: PathBuf,
    /// Аргумент логирования с подставленным путём (защита log4j в старых версиях).
    pub logging_arg: Option<String>,
}

pub struct Installer {
    pub paths: Paths,
    pub endpoints: Endpoints,
    pub downloader: Downloader,
    pub env: Env,
    /// Своя Java вместо рантайма Mojang (настройка профиля). Тогда Java не качается.
    pub java_override: Option<PathBuf>,
}

impl Installer {
    pub fn new(paths: Paths) -> Result<Self> {
        Ok(Self {
            paths,
            endpoints: Endpoints::default(),
            downloader: Downloader::new()?,
            env: Env::current(),
            java_override: None,
        })
    }

    pub fn with_endpoints(mut self, endpoints: Endpoints) -> Self {
        self.endpoints = endpoints;
        self
    }

    /// Список версий. Без сети — последний сохранённый.
    pub async fn version_manifest(&self) -> Result<VersionManifest> {
        let cache = self.paths.cache().join("version_manifest_v2.json");
        match self
            .downloader
            .get_bytes(&self.endpoints.version_manifest)
            .await
        {
            Ok(bytes) => {
                let manifest = parse::<VersionManifest>(&bytes, "список версий")?;
                write_atomic(&cache, &bytes).await?;
                Ok(manifest)
            }
            Err(e) => match tokio::fs::read(&cache).await {
                Ok(bytes) => {
                    tracing::warn!(%e, "нет сети, использую сохранённый список версий");
                    parse(&bytes, "сохранённый список версий")
                }
                Err(_) => Err(e),
            },
        }
    }

    /// JSON версии с учётом наследования (inheritsFrom).
    pub async fn resolve_version(&self, id: &str) -> Result<VersionJson> {
        let mut chain = vec![self.load_version_json(id).await?];
        while let Some(parent) = chain.last().and_then(|v| v.inherits_from.clone()) {
            if chain.len() > 8 {
                return Err(Error::UnknownVersion(format!(
                    "{id}: слишком длинная цепочка наследования"
                )));
            }
            chain.push(self.load_version_json(&parent).await?);
        }
        let mut merged = chain.pop().expect("цепочка не пуста");
        while let Some(child) = chain.pop() {
            merged = child.merge_parent(merged);
        }
        Ok(merged)
    }

    /// versions/<id>/<id>.json: из файла или с серверов Mojang (с проверкой хеша).
    async fn load_version_json(&self, id: &str) -> Result<VersionJson> {
        let path = self.paths.versions().join(id).join(format!("{id}.json"));
        if let Ok(bytes) = tokio::fs::read(&path).await {
            return parse(&bytes, &format!("версию {id}"));
        }
        let manifest = self.version_manifest().await?;
        let entry = manifest
            .find(id)
            .ok_or_else(|| Error::UnknownVersion(id.to_string()))?;
        let mut d = Download::new(&entry.url, &path);
        if let Some(sha1) = &entry.sha1 {
            d = d.sha1(sha1);
        }
        self.downloader.fetch(&d, None).await?;
        let bytes = tokio::fs::read(&path).await.at(&path)?;
        parse(&bytes, &format!("версию {id}"))
    }

    /// Установить всё нужное для запуска версии и вернуть параметры запуска.
    /// `game_dir` нужен только совсем старым версиям (ассеты в resources/).
    pub async fn prepare(
        &self,
        version_id: &str,
        game_dir: &Path,
        on_progress: impl Fn(Progress) + Send + Sync + 'static,
    ) -> Result<Prepared> {
        let on_progress = Arc::new(on_progress);
        on_progress(Progress::stage(Stage::Resolving));
        let version = self.resolve_version(version_id).await?;
        let libraries_dir = self.paths.libraries();
        let assets_root = self.paths.assets();
        let jar_id = version.jar_id().to_string();
        let client_jar = self
            .paths
            .versions()
            .join(&jar_id)
            .join(format!("{jar_id}.jar"));
        let natives_dir = self.paths.versions().join(&version.id).join("natives");

        let mut downloads: Vec<Download> = Vec::new();

        // client.jar
        if let Some(client) = version.downloads.as_ref().and_then(|d| d.client.as_ref()) {
            downloads.push(
                Download::new(&client.url, &client_jar)
                    .sha1(&client.sha1)
                    .size(client.size),
            );
        } else if !client_jar.exists() {
            return Err(Error::UnknownVersion(format!(
                "{version_id}: в описании версии нет client.jar"
            )));
        }

        // Библиотеки и нативы.
        let mut classpath = Vec::new();
        let mut natives = Vec::new();
        for lib in &version.libraries {
            let r = lib.resolve(&self.env, &libraries_dir);
            if let Some(cp) = r.classpath {
                if !classpath.contains(&cp) {
                    classpath.push(cp);
                }
            }
            if let Some(n) = r.native {
                natives.push(n);
            }
            downloads.extend(r.downloads);
        }
        classpath.push(client_jar.clone());

        // Индекс ассетов (маленький, нужен сразу) и сами ассеты.
        let index_ref = version.asset_index.clone().ok_or_else(|| {
            Error::UnknownVersion(format!("{version_id}: в описании версии нет ассетов"))
        })?;
        let index_path = assets_root
            .join("indexes")
            .join(format!("{}.json", index_ref.id));
        let index_dl = Download::new(&index_ref.url, &index_path)
            .sha1(&index_ref.sha1)
            .size(index_ref.size);
        self.downloader.fetch(&index_dl, None).await?;
        let index: AssetIndex = parse(
            &tokio::fs::read(&index_path).await.at(&index_path)?,
            "индекс ассетов",
        )?;
        let mut seen = std::collections::HashSet::new();
        for obj in index.objects.values() {
            if seen.insert(obj.hash.clone()) {
                downloads.push(obj.download(&self.endpoints.resources, &assets_root));
            }
        }

        // Конфиг логирования (в старых версиях закрывает уязвимость Log4Shell).
        let mut logging_arg = None;
        if let Some(client) = version.logging.as_ref().and_then(|l| l.client.as_ref()) {
            let id = client
                .file
                .id
                .clone()
                .unwrap_or_else(|| "client.xml".into());
            let path = assets_root.join("log_configs").join(&id);
            downloads.push(
                Download::new(&client.file.url, &path)
                    .sha1(&client.file.sha1)
                    .size(client.file.size),
            );
            logging_arg = Some(client.argument.replace("${path}", &path.to_string_lossy()));
        }

        // Java.
        let (component, java_major) = version.java_component();
        on_progress(Progress::stage(Stage::Java));
        let java_plan = match &self.java_override {
            Some(_) => None,
            None => Some(
                java::plan(
                    &self.downloader,
                    &self.endpoints.java_runtimes,
                    &self.paths.runtimes(),
                    &component,
                )
                .await?,
            ),
        };
        if let Some(plan) = &java_plan {
            downloads.extend(plan.downloads.iter().cloned());
        }

        // Скачиваем только то, чего нет.
        on_progress(Progress::stage(Stage::Checking));
        let missing = filter_missing(downloads).await;
        let total: u64 = missing.iter().map(|d| d.size.unwrap_or(0)).sum();
        let files = missing.len() as u64;
        if !missing.is_empty() {
            tracing::info!(files, total, "загрузка файлов игры");
            let done = Arc::new(AtomicU64::new(0));
            let cb: OnBytes = {
                let done = done.clone();
                let on_progress = on_progress.clone();
                Arc::new(move |n| {
                    let now = done.fetch_add(n, Ordering::Relaxed) + n;
                    on_progress(Progress::download(now.min(total), total));
                })
            };
            on_progress(Progress::download(0, total));
            self.downloader.fetch_all(missing, Some(cb)).await?;
        }

        on_progress(Progress::stage(Stage::Unpacking));
        if let Some(plan) = &java_plan {
            plan.finish().await?;
        }
        extract_natives(&natives, &natives_dir).await?;

        let mut game_assets = assets_root.clone();
        if index.is_virtual || index.map_to_resources {
            let target = if index.map_to_resources {
                game_dir.join("resources")
            } else {
                assets_root.join("virtual").join(&index_ref.id)
            };
            copy_named_assets(&index, &assets_root, &target).await?;
            game_assets = target;
        }

        Ok(Prepared {
            java_major,
            java: match (&self.java_override, java_plan) {
                (Some(java), _) => java.clone(),
                (None, Some(plan)) => plan.java,
                (None, None) => unreachable!("план Java есть, когда своей Java нет"),
            },
            client_jar,
            classpath,
            natives_dir,
            libraries_dir,
            assets_root,
            asset_index_id: version.assets.clone().unwrap_or(index_ref.id),
            game_assets,
            logging_arg,
            version,
        })
    }
}

async fn filter_missing(downloads: Vec<Download>) -> Vec<Download> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for d in downloads {
        if seen.insert(d.path.clone()) && !Downloader::is_complete(&d).await {
            out.push(d);
        }
    }
    out
}

/// Распаковать нативные библиотеки (.dll/.so/.dylib) в папку версии.
async fn extract_natives(natives: &[(PathBuf, Vec<String>)], dir: &Path) -> Result<()> {
    if natives.is_empty() {
        return Ok(());
    }
    let natives = natives.to_vec();
    let dir = dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&dir).at(&dir)?;
        for (jar, exclude) in &natives {
            let file = std::fs::File::open(jar).at(jar)?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| Error::Zip {
                path: jar.clone(),
                message: e.to_string(),
            })?;
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).map_err(|e| Error::Zip {
                    path: jar.clone(),
                    message: e.to_string(),
                })?;
                let name = entry
                    .name()
                    .map_err(|e| Error::Zip {
                        path: jar.clone(),
                        message: e.to_string(),
                    })?
                    .to_string();
                if entry.is_dir()
                    || exclude.iter().any(|ex| name.starts_with(ex.as_str()))
                    || name.starts_with("META-INF/")
                {
                    continue;
                }
                // Только сами библиотеки; путь внутри архива не доверяем (zip-slip).
                let Some(file_name) = Path::new(&name).file_name() else {
                    continue;
                };
                let out = dir.join(file_name);
                if out.exists() {
                    continue;
                }
                let tmp = out.with_extension("tmp");
                let mut f = std::fs::File::create(&tmp).at(&tmp)?;
                std::io::copy(&mut entry, &mut f).at(&tmp)?;
                drop(f);
                std::fs::rename(&tmp, &out).at(&out)?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| Error::Launch(e.to_string()))?
}

/// Старые версии ищут ассеты по именам, а не по хешам — раскладываем копии.
async fn copy_named_assets(index: &AssetIndex, assets_root: &Path, target: &Path) -> Result<()> {
    for (name, obj) in &index.objects {
        // Имена из индекса Mojang; на всякий случай не даём выйти за пределы папки.
        if name.split(['/', '\\']).any(|p| p == "..") {
            continue;
        }
        let dest = target.join(name);
        if tokio::fs::metadata(&dest)
            .await
            .is_ok_and(|m| m.len() == obj.size)
        {
            continue;
        }
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.at(parent)?;
        }
        let src = assets_root.join("objects").join(obj.rel_path());
        tokio::fs::copy(&src, &dest).await.at(&dest)?;
    }
    Ok(())
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8], what: &str) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|source| Error::Json {
        what: what.to_string(),
        source,
    })
}

async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await.at(dir)?;
    }
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, bytes).await.at(&tmp)?;
    tokio::fs::rename(&tmp, path).await.at(path)
}
