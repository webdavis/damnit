use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: String,
    pub authorization: Option<String>,
}

impl Seen {
    pub fn form(&self) -> HashMap<String, String> {
        decoded_fields(&self.body)
    }

    pub fn json(&self) -> serde_json::Value {
        if let Ok(value) = serde_json::from_str(&self.body) {
            return value;
        }
        self.form()
            .into_iter()
            .map(|(name, value)| {
                let parsed =
                    serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
                (name, parsed)
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
}

pub(super) fn read(reader: &mut BufReader<TcpStream>) -> Option<Seen> {
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.is_empty() {
        return None;
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
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
    Some(Seen {
        method,
        path: path.to_string(),
        query: query.to_string(),
        body: String::from_utf8_lossy(&raw).into_owned(),
        authorization,
    })
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
