use dam_domain::{Field, Object};
use serde_json::Value;

use crate::view::document;

pub(super) fn cells(object: &Object, fields: &[Field]) -> impl Iterator<Item = String> {
    let doc = serde_json::to_value(document(object)).unwrap_or_default();
    fields
        .iter()
        .filter(|f| !matches!(f, Field::Subject | Field::Path))
        .map(move |f| format!("{f}: {}", text(found(&doc, f.as_str()))))
}

fn found<'d>(doc: &'d Value, name: &str) -> &'d Value {
    ["task", "event"]
        .iter()
        .find_map(|side| doc.get(side)?.get(name))
        .or_else(|| doc.get(name))
        .unwrap_or(&Value::Null)
}

fn text(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) if s.trim().is_empty() => "-".to_string(),
        Value::String(s) => s.split_whitespace().collect::<Vec<_>>().join(" "),
        Value::Array(items) if items.is_empty() => "-".to_string(),
        Value::Array(items) => items.iter().map(text).collect::<Vec<_>>().join(", "),
        Value::Object(map) => map.values().map(text).collect::<Vec<_>>().join(" "),
        other => other.to_string(),
    }
}
