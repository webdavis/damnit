// Each test binary compiles this module and uses the part of it that test
// needs, so the rest is unused there.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: String,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl Reply {
    pub fn json(status: u16, body: serde_json::Value) -> Reply {
        Reply {
            status,
            body: body.to_string(),
            headers: vec![],
        }
    }
    pub fn with_header(mut self, name: &str, value: &str) -> Reply {
        self.headers.push((name.into(), value.into()));
        self
    }
}

pub struct Loopback {
    pub base: String,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

impl Loopback {
    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// `routes` maps "METHOD /path", the query left off, to the replies it gives
/// in order; the last one repeats. An unmatched request gets 404.
pub fn serve(routes: HashMap<&'static str, Vec<Reply>>) -> Loopback {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("bind: {e}"));
    let base = format!(
        "http://{}",
        listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
    );
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let mut served: HashMap<String, usize> = HashMap::new();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let target = parts.next().unwrap_or("").to_string();
            let (path, query) = target
                .split_once('?')
                .map_or((target.clone(), String::new()), |(p, q)| {
                    (p.to_string(), q.to_string())
                });
            let mut length = 0usize;
            let mut authorization = None;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).is_err() || header.trim().is_empty() {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
                if lower.starts_with("authorization:") {
                    authorization = header.split_once(':').map(|(_, v)| v.trim().to_string());
                }
            }
            let mut raw = vec![0u8; length];
            let _ = reader.read_exact(&mut raw);
            let body = String::from_utf8_lossy(&raw).into_owned();
            if let Ok(mut l) = log.lock() {
                l.push(Seen {
                    method: method.clone(),
                    path: path.clone(),
                    query,
                    body,
                    authorization,
                });
            }
            let key = format!("{method} {path}");
            let reply = routes.get(key.as_str()).and_then(|queue| {
                let n = served.entry(key.clone()).or_insert(0);
                let reply = queue.get(*n).or_else(|| queue.last()).cloned();
                *n += 1;
                reply
            });
            let reply =
                reply.unwrap_or_else(|| Reply::json(404, serde_json::json!({"error": "no route"})));
            let mut head = format!(
                "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
                reply.status,
                reply.body.len()
            );
            for (name, value) in &reply.headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            let _ = reader
                .get_mut()
                .write_all(format!("{head}\r\n{}", reply.body).as_bytes());
        }
    });
    Loopback { base, seen }
}
