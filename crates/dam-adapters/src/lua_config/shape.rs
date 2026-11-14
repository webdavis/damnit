use mlua::{Table, Value};

pub(super) fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
        _ => None,
    }
}

pub(super) fn list(value: &Value) -> Option<Vec<Value>> {
    let Value::Table(table) = value else {
        return None;
    };
    let items = table
        .sequence_values::<Value>()
        .collect::<mlua::Result<Vec<_>>>()
        .ok()?;
    (table.pairs::<Value, Value>().count() == items.len()).then_some(items)
}

pub(super) fn map_entries(table: &Table) -> Option<Vec<(String, Value)>> {
    let mut entries = table
        .pairs::<Value, Value>()
        .map(|pair| {
            let (key, value) = pair.ok()?;
            Some((text(&key)?, value))
        })
        .collect::<Option<Vec<_>>>()?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Some(entries)
}
