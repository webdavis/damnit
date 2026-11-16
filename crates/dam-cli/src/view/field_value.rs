use serde_json::Value;

pub(crate) fn field_value<'d>(doc: &'d Value, name: &str) -> &'d Value {
    ["task", "event"]
        .iter()
        .find_map(|side| doc.get(side)?.get(name))
        .or_else(|| doc.get(name))
        .unwrap_or(&Value::Null)
}
