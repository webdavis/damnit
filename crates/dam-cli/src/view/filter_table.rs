use dam_domain::{FilterTable, TableValue};
use serde_json::{Map, Value};

use crate::args::selection::flag_of;
use crate::error::quoted;

const ANY: &str = "any";

pub(crate) fn table_json(t: &FilterTable) -> Value {
    let mut fields: Map<String, Value> = t
        .fields
        .iter()
        .map(|(key, value)| (key.clone(), value_json(value)))
        .collect();
    if !t.any.is_empty() {
        fields.insert(ANY.into(), t.any.iter().map(table_json).collect());
    }
    Value::Object(fields)
}

fn value_json(value: &TableValue) -> Value {
    match value {
        TableValue::One(text) => Value::String(text.clone()),
        TableValue::Many(texts) => texts.iter().cloned().map(Value::String).collect(),
    }
}

pub(crate) fn flags_of(t: &FilterTable) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for (key, value) in &t.fields {
        for text in value.texts() {
            words.push(format!("--{} {}", flag_of(key), quoted(text)));
        }
    }
    for alternative in &t.any {
        let [(key, TableValue::One(text))] = alternative.fields.iter().collect::<Vec<_>>()[..]
        else {
            return None;
        };
        if !alternative.any.is_empty() {
            return None;
        }
        let condition = format!("{}={text}", flag_of(key));
        words.push(format!("--any {}", quoted(&condition)));
    }
    Some(words.join(" "))
}
