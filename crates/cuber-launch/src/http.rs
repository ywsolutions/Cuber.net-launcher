//! Загрузка файлов: докачка после обрыва, проверка целостности, повторы, параллельность.
//!
//! Файл качается во временный `<имя>.part`. Если загрузка оборвалась, при следующей
//! попытке (в том числе после перезапуска лаунчера) продолжается с места обрыва
//! через HTTP Range. Готовый файл проверяется по размеру и SHA-1 и только после
//! этого атомарно переименовывается в итоговый — битый файл никогда не попадёт
//! на место настоящего.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::{StreamExt, TryStreamExt};
use reqwest::header::{CONTENT_RANGE, RANGE};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;

use crate::error::{Error, IoContext, Result};

const USER_AGENT: &str = concat!(
    "Cuber.net/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/ywsolutions/Cuber.net-launcher)"
);

/// Файл, который нужно скачать.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    /// Ожидаемый SHA-1 (hex). Если известен — файл обязательно проверяется.
    pub sha1: Option<String>,
    /// Ожидаемый размер, байт.
    pub size: Option<u64>,
}

impl Download {
    pub fn new(url: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            path: path.into(),
            sha1: None,
            size: None,
        }
    }

    pub fn sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into().to_ascii_lowercase());
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    fn part_path(&self) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(".part");
        self.path.with_file_name(name)
    }
}

/// Колбэк прогресса: сколько новых байт записано.
pub type OnBytes = Arc<dyn Fn(u64) + Send + Sync>;

#[derive(Clone)]
pub struct Downloader {
    client: reqwest::Client,
    concurrency: usize,
    retries: u32,
}

impl Downloader {
    pub fn new() -> Result<Self> {
        // rustls без встроенного провайдера: ставим ring один раз на процесс.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(|source| Error::Http {
                url: "(клиент)".into(),
                source,
            })?;
        Ok(Self {
            client,
            concurrency: 16,
            retries: 4,
        })
    }

    pub fn with_concurrency(mut self, n: usize) -> Self {
        self.concurrency = n.max(1);
        self
    }

    /// GET и разбор JSON, с повторами.
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let bytes = self.get_bytes(url).await?;
        serde_json::from_slice(&bytes).map_err(|source| Error::Json {
            what: url.to_string(),
            source,
        })
    }

    /// GET целиком в память, с повторами.
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        self.retry(|| async {
            let resp = self.send(self.client.get(url), url).await?;
            let bytes = resp.bytes().await.map_err(|source| Error::Http {
                url: url.to_string(),
                source,
            })?;
            Ok(bytes.to_vec())
        })
        .await
    }

    /// Уже скачан ли файл. Проверяется по размеру (быстро); SHA-1 — только если
    /// размер неизвестен. Полную проверку хеша делает [`Downloader::verify`].
    pub async fn is_complete(d: &Download) -> bool {
        let Ok(meta) = tokio::fs::metadata(&d.path).await else {
            return false;
        };
        match (d.size, &d.sha1) {
            (Some(size), _) => meta.len() == size,
            (None, Some(sha1)) => sha1_file(&d.path).await.ok().as_deref() == Some(sha1.as_str()),
            (None, None) => true,
        }
    }

    /// Полная проверка готового файла по SHA-1 и размеру.
    pub async fn verify(d: &Download) -> Result<()> {
        check(&d.path, d).await
    }

    /// Скачать один файл (с докачкой и повторами), если его ещё нет.
    pub async fn fetch(&self, d: &Download, on_bytes: Option<OnBytes>) -> Result<()> {
        if Self::is_complete(d).await {
            return Ok(());
        }
        if let Some(dir) = d.path.parent() {
            tokio::fs::create_dir_all(dir).await.at(dir)?;
        }
        self.retry(|| self.fetch_once(d, on_bytes.clone())).await
    }

    /// Скачать набор файлов параллельно. Ошибка любого файла прерывает весь набор
    /// (уже скачанное сохраняется и при повторе не качается заново).
    pub async fn fetch_all(&self, items: Vec<Download>, on_bytes: Option<OnBytes>) -> Result<()> {
        futures::stream::iter(items.into_iter().map(|d| {
            let on_bytes = on_bytes.clone();
            async move { self.fetch(&d, on_bytes).await }
        }))
        .buffer_unordered(self.concurrency)
        .try_collect::<Vec<()>>()
        .await?;
        Ok(())
    }

    async fn fetch_once(&self, d: &Download, on_bytes: Option<OnBytes>) -> Result<()> {
        let part = d.part_path();
        let mut have = tokio::fs::metadata(&part)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        // .part уже полный (например, упали между загрузкой и переименованием).
        if have > 0 && d.size == Some(have) {
            return finish(&part, d).await;
        }
        if d.size.is_some_and(|size| have > size) {
            let _ = tokio::fs::remove_file(&part).await;
            have = 0;
        }

        let mut req = self.client.get(&d.url);
        if have > 0 {
            req = req.header(RANGE, format!("bytes={have}-"));
        }
        let resp = req.send().await.map_err(|source| Error::Http {
            url: d.url.clone(),
            source,
        })?;

        let append = match resp.status() {
            StatusCode::PARTIAL_CONTENT if have > 0 && range_starts_at(&resp, have) => true,
            StatusCode::RANGE_NOT_SATISFIABLE if have > 0 => {
                // Сервер считает, что докачивать нечего — проверим, что есть.
                return match finish(&part, d).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        let _ = tokio::fs::remove_file(&part).await;
                        Err(e)
                    }
                };
            }
            s if s.is_success() => false,
            s => {
                return Err(Error::Status {
                    url: d.url.clone(),
                    status: s.as_u16(),
                })
            }
        };

        let mut file = if append {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&part)
                .await
                .at(&part)?
        } else {
            tokio::fs::File::create(&part).await.at(&part)?
        };

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|source| Error::Http {
                url: d.url.clone(),
                source,
            })?;
            file.write_all(&chunk).await.at(&part)?;
            if let Some(cb) = &on_bytes {
                cb(chunk.len() as u64);
            }
        }
        file.flush().await.at(&part)?;
        drop(file);

        match finish(&part, d).await {
            Ok(()) => Ok(()),
            Err(e) => {
                // Битый результат не докачиваем — следующая попытка начнёт заново.
                let _ = tokio::fs::remove_file(&part).await;
                Err(e)
            }
        }
    }

    async fn send(&self, req: reqwest::RequestBuilder, url: &str) -> Result<reqwest::Response> {
        let resp = req.send().await.map_err(|source| Error::Http {
            url: url.to_string(),
            source,
        })?;
        if !resp.status().is_success() {
            return Err(Error::Status {
                url: url.to_string(),
                status: resp.status().as_u16(),
            });
        }
        Ok(resp)
    }

    async fn retry<T, F, Fut>(&self, mut op: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        loop {
            match op().await {
                Ok(v) => return Ok(v),
                Err(e) if e.is_retryable() && attempt < self.retries => {
                    attempt += 1;
                    let delay = Duration::from_millis(400 * 2u64.pow(attempt - 1));
                    tracing::debug!(%e, attempt, "повтор загрузки");
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

fn range_starts_at(resp: &reqwest::Response, offset: u64) -> bool {
    // Content-Range: bytes 100-999/1000
    resp.headers()
        .get(CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("bytes "))
        .and_then(|v| v.split('-').next())
        .and_then(|v| v.parse::<u64>().ok())
        == Some(offset)
}

/// Проверить .part и переименовать в итоговый файл.
async fn finish(part: &Path, d: &Download) -> Result<()> {
    check(part, d).await?;
    tokio::fs::rename(part, &d.path).await.at(&d.path)
}

async fn check(path: &Path, d: &Download) -> Result<()> {
    if let Some(size) = d.size {
        let actual = tokio::fs::metadata(path).await.at(path)?.len();
        if actual != size {
            return Err(Error::Size {
                path: d.path.clone(),
                expected: size,
                actual,
            });
        }
    }
    if let Some(expected) = &d.sha1 {
        let actual = sha1_file(path).await?;
        if &actual != expected {
            return Err(Error::Hash {
                path: d.path.clone(),
                expected: expected.clone(),
                actual,
            });
        }
    }
    Ok(())
}

/// SHA-1 файла (hex, нижний регистр).
pub async fn sha1_file(path: &Path) -> Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut file = std::fs::File::open(&path).at(&path)?;
        let mut hasher = Sha1::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buf).at(&path)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    })
    .await
    .map_err(|e| Error::Launch(e.to_string()))?
}
