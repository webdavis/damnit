use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub body: serde_json::Value,
    pub authorization: Option<String>,
}

pub struct Loopback {
    pub base: String,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

/// `routes` maps "METHOD /path" to (status, JSON body). Unmatched requests get 404.
pub fn serve(routes: HashMap<&'static str, (u16, serde_json::Value)>) -> Loopback {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("bind: {e}"));
    let base = format!(
        "http://{}",
        listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
    );
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let path = parts.next().unwrap_or("").to_string();
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
                if let Some(v) = header
                    .strip_prefix("Authorization:")
                    .or_else(|| header.strip_prefix("authorization:"))
                {
                    authorization = Some(v.trim().to_string());
                }
            }
            let mut raw = vec![0u8; length];
            let _ = reader.read_exact(&mut raw);
            let body = serde_json::from_slice(&raw).unwrap_or(serde_json::Value::Null);
            if let Ok(mut l) = log.lock() {
                l.push(Seen {
                    method: method.clone(),
                    path: path.clone(),
                    body,
                    authorization,
                });
            }
            let key = format!("{method} {path}");
            let (status, payload) = routes
                .get(key.as_str())
                .cloned()
                .unwrap_or((404, serde_json::json!({"error": "no route"})));
            let text = payload.to_string();
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                text.len()
            );
            let _ = reader.get_mut().write_all(response.as_bytes());
        }
    });
    Loopback { base, seen }
}
