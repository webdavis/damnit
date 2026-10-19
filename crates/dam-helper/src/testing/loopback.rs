mod request;

use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub use request::{Seen, decoded_fields};

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

    pub fn with_header(mut self, name: &str, value: impl Into<String>) -> Reply {
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
    let mut served: HashMap<String, usize> = HashMap::new();
    serve_with(move |request| {
        let key = format!("{} {}", request.method, request.path);
        replies_in_order_by_method_and_path
            .get(key.as_str())
            .and_then(|replies| next_reply_repeating_the_last(replies, &mut served, &key))
            .unwrap_or_else(not_found)
    })
}

pub fn serve_with(mut responder: impl FnMut(&Seen) -> Reply + Send + 'static) -> Loopback {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("bind: {e}"));
    let base = format!(
        "http://{}",
        listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
    );
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = stream.set_nodelay(true);
            let mut reader = BufReader::new(stream);
            let Some(request) = request::read(&mut reader) else {
                continue;
            };
            if let Ok(mut l) = log.lock() {
                l.push(request.clone());
            }
            answer(reader.get_mut(), &responder(&request));
        }
    });
    Loopback { base, seen }
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
