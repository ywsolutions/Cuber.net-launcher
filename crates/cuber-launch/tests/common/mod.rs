//! Крошечный HTTP-сервер для тестов: отдаёт файлы из памяти, понимает Range
//! и умеет один раз оборвать соединение посреди файла (имитация обрыва сети).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default)]
pub struct State {
    pub files: HashMap<String, Vec<u8>>,
    /// Пути, которые при первом запросе обрываются после половины файла.
    pub cut_once: Vec<String>,
    /// Журнал запросов: (путь, заголовок Range).
    pub log: Vec<(String, Option<String>)>,
}

pub struct Server {
    pub base: String,
    pub state: Arc<Mutex<State>>,
}

impl Server {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State::default()));
        let st = state.clone();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let st = st.clone();
                thread::spawn(move || handle(stream, st));
            }
        });
        Self { base, state }
    }

    pub fn put(&self, path: &str, bytes: impl Into<Vec<u8>>) -> String {
        self.state
            .lock()
            .unwrap()
            .files
            .insert(path.to_string(), bytes.into());
        format!("{}{}", self.base, path)
    }

    #[allow(dead_code)]
    pub fn requests(&self, path: &str) -> Vec<Option<String>> {
        self.state
            .lock()
            .unwrap()
            .log
            .iter()
            .filter(|(p, _)| p == path)
            .map(|(_, r)| r.clone())
            .collect()
    }
}

fn handle(mut stream: TcpStream, state: Arc<Mutex<State>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();
    let mut range = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("range:") {
            range = Some(v.trim().to_string());
        }
    }

    let (body, cut) = {
        let mut s = state.lock().unwrap();
        s.log.push((path.clone(), range.clone()));
        let body = s.files.get(&path).cloned();
        let cut = if let Some(i) = s.cut_once.iter().position(|p| p == &path) {
            s.cut_once.remove(i);
            true
        } else {
            false
        };
        (body, cut)
    };

    let Some(body) = body else {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return;
    };

    let start = range
        .as_deref()
        .and_then(|r| r.strip_prefix("bytes="))
        .and_then(|r| r.split('-').next())
        .and_then(|n| n.parse::<usize>().ok());

    match start {
        Some(s) if s >= body.len() => {
            let _ = write!(
                stream,
                "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                body.len()
            );
        }
        Some(s) => {
            let part = &body[s..];
            let _ = write!(
                stream,
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
                part.len(),
                s,
                body.len() - 1,
                body.len()
            );
            let _ = stream.write_all(part);
        }
        None => {
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n",
                body.len()
            );
            if cut {
                let _ = stream.write_all(&body[..body.len() / 2]);
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return;
            }
            let _ = stream.write_all(&body);
        }
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(bytes))
}
