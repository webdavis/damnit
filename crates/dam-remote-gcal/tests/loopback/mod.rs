#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: String,
    pub decoded_form: HashMap<String, String>,
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

pub fn serve(replies_in_order_by_method_and_path: HashMap<&'static str, Vec<Reply>>) -> Loopback {
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
            let Some(request) = read_request(&mut reader) else {
                continue;
            };
            let key = format!("{} {}", request.method, request.path);
            if let Ok(mut l) = log.lock() {
                l.push(request);
            }
            let reply = replies_in_order_by_method_and_path
                .get(key.as_str())
                .and_then(|replies| next_reply_repeating_the_last(replies, &mut served, &key))
                .unwrap_or_else(not_found);
            answer(reader.get_mut(), &reply);
        }
    });
    Loopback { base, seen }
}

fn read_request(reader: &mut BufReader<TcpStream>) -> Option<Seen> {
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.is_empty() {
        return None;
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
    Some(Seen {
        method,
        path,
        query,
        decoded_form: decoded_fields(&body),
        body,
        authorization,
    })
}

fn next_reply_repeating_the_last(
    replies: &[Reply],
    served: &mut HashMap<String, usize>,
    key: &str,
) -> Option<Reply> {
    let n = served.entry(key.to_string()).or_insert(0);
    let reply = replies.get(*n).or_else(|| replies.last()).cloned();
    *n += 1;
    reply
}

fn not_found() -> Reply {
    Reply::json(404, serde_json::json!({"error": "no route"}))
}

fn answer(stream: &mut TcpStream, reply: &Reply) {
    let mut head = format!(
        "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        reply.status,
        reply.body.len()
    );
    for (name, value) in &reply.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    let _ = stream.write_all(format!("{head}\r\n{}", reply.body).as_bytes());
}

pub fn decoded_fields(form_or_query: &str) -> HashMap<String, String> {
    form_or_query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, value)| (decoded(name), decoded(value)))
        .collect()
}

fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .filter(|h| h.iter().all(u8::is_ascii_hexdigit));
        match (bytes[i], hex) {
            (b'%', Some(h)) => {
                out.push(u8::from_str_radix(&String::from_utf8_lossy(h), 16).unwrap_or(b'?'));
                i += 3;
            }
            (b'+', _) => {
                out.push(b' ');
                i += 1;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
