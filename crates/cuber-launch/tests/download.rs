//! Загрузчик: докачка после обрыва, отказ от битых файлов, пропуск готовых.

mod common;

use common::{sha1_hex, Server};
use cuber_launch::http::{Download, Downloader};

fn payload(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 31 % 251) as u8).collect()
}

#[tokio::test]
async fn resumes_after_connection_drop() {
    let server = Server::start();
    let data = payload(512 * 1024);
    let url = server.put("/big.bin", data.clone());
    server
        .state
        .lock()
        .unwrap()
        .cut_once
        .push("/big.bin".into());

    let tmp = tempfile::tempdir().unwrap();
    let d = Download::new(url, tmp.path().join("big.bin"))
        .sha1(sha1_hex(&data))
        .size(data.len() as u64);

    Downloader::new().unwrap().fetch(&d, None).await.unwrap();

    assert_eq!(std::fs::read(&d.path).unwrap(), data);
    assert!(!tmp.path().join("big.bin.part").exists());
    let reqs = server.requests("/big.bin");
    assert_eq!(reqs.len(), 2, "ожидался обрыв и одна докачка: {reqs:?}");
    // Вторая попытка продолжила с середины, а не качала заново.
    assert_eq!(
        reqs[1].as_deref(),
        Some(&*format!("bytes={}-", data.len() / 2))
    );
}

#[tokio::test]
async fn resumes_leftover_part_file_from_previous_run() {
    let server = Server::start();
    let data = payload(100_000);
    let url = server.put("/f.bin", data.clone());
    let tmp = tempfile::tempdir().unwrap();
    // Как будто лаунчер закрыли посреди загрузки.
    std::fs::write(tmp.path().join("f.bin.part"), &data[..30_000]).unwrap();

    let d = Download::new(url, tmp.path().join("f.bin"))
        .sha1(sha1_hex(&data))
        .size(data.len() as u64);
    Downloader::new().unwrap().fetch(&d, None).await.unwrap();

    assert_eq!(std::fs::read(&d.path).unwrap(), data);
    assert_eq!(
        server.requests("/f.bin"),
        vec![Some("bytes=30000-".to_string())]
    );
}

#[tokio::test]
async fn rejects_corrupted_file() {
    let server = Server::start();
    let url = server.put("/bad.bin", b"not what we expected".to_vec());
    let tmp = tempfile::tempdir().unwrap();
    let d = Download::new(url, tmp.path().join("bad.bin")).sha1(sha1_hex(b"real content"));

    let err = Downloader::new()
        .unwrap()
        .fetch(&d, None)
        .await
        .unwrap_err();
    assert!(matches!(err, cuber_launch::Error::Hash { .. }), "{err}");
    // Битый файл не попадает на место настоящего.
    assert!(!d.path.exists());
    assert!(!tmp.path().join("bad.bin.part").exists());
}

#[tokio::test]
async fn skips_complete_files() {
    let server = Server::start();
    let data = payload(1000);
    let url = server.put("/ok.bin", data.clone());
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("ok.bin"), &data).unwrap();

    let d = Download::new(url, tmp.path().join("ok.bin"))
        .sha1(sha1_hex(&data))
        .size(1000);
    Downloader::new().unwrap().fetch(&d, None).await.unwrap();
    assert!(server.requests("/ok.bin").is_empty());
}
