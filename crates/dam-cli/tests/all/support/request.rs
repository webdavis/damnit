use std::io::Write;

use serde_json::Value;

use super::sandbox::Sandbox;

pub fn request_file(sb: &Sandbox, document: &str) -> String {
    let path = sb.dir.path().join("request.json");
    std::fs::write(&path, document).unwrap();
    path.display().to_string()
}

pub fn by_request(sb: &Sandbox, command: &[&str], document: &Value) -> std::process::Output {
    let path = request_file(sb, &document.to_string());
    sb.output(&[command, &["--json", "--request", &path]].concat())
}

pub fn fed(sb: &Sandbox, args: &[&str], input: &str) -> std::process::Output {
    let mut child = sb.spawn_with_stdin(args, std::process::Stdio::piped());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

pub fn answered(out: &std::process::Output) -> Value {
    assert!(out.status.success(), "{out:?}");
    without_oids_hashes_or_times(serde_json::from_slice(&out.stdout).unwrap())
}

pub fn without_oids_hashes_or_times(value: Value) -> Value {
    match value {
        Value::String(text) if is_hex(&text) => Value::String("<hex>".into()),
        Value::String(text) if text.parse::<jiff::Timestamp>().is_ok() => {
            Value::String("<time>".into())
        }
        Value::Array(items) => items
            .into_iter()
            .map(without_oids_hashes_or_times)
            .collect(),
        Value::Object(fields) => fields
            .into_iter()
            .map(|(key, value)| (key, without_oids_hashes_or_times(value)))
            .collect(),
        other => other,
    }
}

fn is_hex(text: &str) -> bool {
    text.len() >= 7 && text.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn usage_refusal(out: &std::process::Output) -> String {
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let document: Value = serde_json::from_slice(&out.stderr)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stderr)));
    assert_eq!(document["error"]["kind"], "usage", "{document}");
    document["error"]["message"].as_str().unwrap().to_string()
}
