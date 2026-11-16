use std::fmt::Write;

use serde_json::Value;

const WORDS_READ_AS_OTHER_TYPES: [&str; 12] = [
    "true", "false", "yes", "no", "on", "off", "y", "n", "null", "nan", "inf", "infinity",
];

const PLAIN_PUNCTUATION: [char; 9] = [' ', '_', '-', '.', '/', '@', '+', '(', ')'];

pub(crate) fn encode(value: &Value) -> String {
    let mut out = String::new();
    block(&mut out, value, 0);
    out
}

fn block(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, value) in map {
                push_indent(out, indent);
                out.push_str(&string(key));
                out.push(':');
                nested(out, value, indent + 2);
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for item in items {
                let mut inner = String::new();
                block(&mut inner, item, indent + 2);
                push_indent(out, indent);
                out.push_str("- ");
                out.push_str(&inner[indent + 2..]);
            }
        }
        _ => {
            push_indent(out, indent);
            out.push_str(&inline(value));
            out.push('\n');
        }
    }
}

fn nested(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            out.push('\n');
            block(out, value, indent);
        }
        Value::Array(items) if !items.is_empty() => {
            out.push('\n');
            block(out, value, indent);
        }
        _ => {
            out.push(' ');
            out.push_str(&inline(value));
            out.push('\n');
        }
    }
}

fn push_indent(out: &mut String, indent: usize) {
    out.extend(std::iter::repeat_n(' ', indent));
}

fn inline(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => string(s),
        Value::Array(_) => "[]".into(),
        Value::Object(_) => "{}".into(),
    }
}

fn string(text: &str) -> String {
    if reads_as_itself(text) {
        text.to_string()
    } else {
        quoted(text)
    }
}

fn reads_as_itself(text: &str) -> bool {
    let starts_with_a_letter = text.chars().next().is_some_and(char::is_alphabetic);
    starts_with_a_letter
        && !text.ends_with(' ')
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || PLAIN_PUNCTUATION.contains(&c))
        && !WORDS_READ_AS_OTHER_TYPES.contains(&text.to_lowercase().as_str())
}

fn quoted(text: &str) -> String {
    let mut out = String::from('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}') => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests;
