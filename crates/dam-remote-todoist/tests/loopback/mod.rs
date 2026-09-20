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
    pub body: serde_json::Value,
    pub authorization: Option<String>,
}

pub struct Loopback {
    pub base: String,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

/// One answer: a status, any extra headers, and a JSON body.
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: serde_json::Value,
}

impl Reply {
    pub fn new(status: u16, body: serde_json::Value) -> Reply {
        Reply {
            status,
            headers: vec![],
            body,
        }
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Reply {
        self.headers.push((name, value.into()));
        self
    }
}

/// `routes` maps "METHOD /path" to (status, JSON body). Unmatched requests get 404.
pub fn serve(routes: HashMap<&'static str, (u16, serde_json::Value)>) -> Loopback {
    serve_with(move |seen| {
        let key = format!("{} {}", seen.method, seen.path);
        match routes.get(key.as_str()) {
            Some((status, body)) => Reply::new(*status, body.clone()),
            None => Reply::new(404, serde_json::json!({"error": "no route"})),
        }
    })
}

/// Answers every request with whatever `responder` decides, so a test can hold
/// state across requests the way the real service does.
pub fn serve_with(responder: impl Fn(&Seen) -> Reply + Send + 'static) -> Loopback {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("bind: {e}"));
    let base = format!(
        "http://{}",
        listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
    );
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            // ureq already sets TCP_NODELAY on its side; match it here so neither side
            // waits on Nagle's algorithm for a response this small.
            let _ = stream.set_nodelay(true);
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
            let request = Seen {
                method,
                path,
                body: decode(&raw),
                authorization,
            };
            if let Ok(mut l) = log.lock() {
                l.push(request.clone());
            }
            let reply = responder(&request);
            let text = reply.body.to_string();
            let extra: String = reply
                .headers
                .iter()
                .map(|(k, v)| format!("{k}: {v}\r\n"))
                .collect();
            let status = reply.status;
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{text}",
                text.len()
            );
            let _ = reader.get_mut().write_all(response.as_bytes());
        }
    });
    Loopback { base, seen }
}

/// A JSON body as itself, or a form body as an object of its decoded fields,
/// each field parsed as JSON when it is JSON. Todoist's sync endpoint takes
/// `commands` as a form field holding a JSON array.
fn decode(raw: &[u8]) -> serde_json::Value {
    if let Ok(v) = serde_json::from_slice(raw) {
        return v;
    }
    let Ok(text) = std::str::from_utf8(raw) else {
        return serde_json::Value::Null;
    };
    let mut fields = serde_json::Map::new();
    for pair in text.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let decoded = form_decode(value);
        let parsed = serde_json::from_str(&decoded).unwrap_or(serde_json::Value::String(decoded));
        fields.insert(form_decode(key), parsed);
    }
    serde_json::Value::Object(fields)
}

fn form_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match u8::from_str_radix(&text[i + 1..i + 3], 16) {
                    Ok(b) => out.push(b),
                    Err(_) => out.push(b'%'),
                }
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
