use std::collections::BTreeMap;

use dam_application::SettingValue;
use dam_domain::{FilterTable, TableValue};
use mlua::{Table, Value};

pub(super) type Flat = BTreeMap<String, Shown>;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Shown {
    Text(String),
    Other(String),
}

pub(super) fn flatten(table: Option<&Table>) -> Flat {
    let mut flat = Flat::new();
    if let Some(table) = table {
        add_table(&mut flat, "", table);
    }
    flat
}

pub(super) fn flatten_value(key: &str, value: &SettingValue) -> Flat {
    match value {
        SettingValue::Text(text) => Flat::from([(key.to_string(), Shown::Text(text.clone()))]),
        SettingValue::Flag(flag) => Flat::from([(key.to_string(), Shown::Other(flag.to_string()))]),
        SettingValue::Count(count) => {
            Flat::from([(key.to_string(), Shown::Other(count.to_string()))])
        }
        SettingValue::List(items) => listed(key, items),
        SettingValue::Table(table) => flatten_table(key, table),
    }
}

fn listed(key: &str, items: &[String]) -> Flat {
    items
        .iter()
        .enumerate()
        .map(|(at, item)| (format!("{key}.{}", at + 1), Shown::Text(item.clone())))
        .collect()
}

fn flatten_table(key: &str, table: &FilterTable) -> Flat {
    let mut flat = Flat::new();
    for (field, value) in &table.fields {
        let name = format!("{key}.{field}");
        match value {
            TableValue::One(text) => {
                flat.insert(name, Shown::Text(text.clone()));
            }
            TableValue::Many(texts) => flat.extend(listed(&name, texts)),
        }
    }
    for (at, alternative) in table.any.iter().enumerate() {
        flat.extend(flatten_table(&format!("{key}.any.{}", at + 1), alternative));
    }
    flat
}

pub(super) fn under(flat: &Flat, key: &str) -> Flat {
    let prefix = format!("{key}.");
    flat.iter()
        .filter(|(name, _)| *name == key || name.starts_with(&prefix))
        .map(|(name, shown)| (name.clone(), shown.clone()))
        .collect()
}

pub(super) fn shown_text(flat: &Flat) -> Option<String> {
    let texts: Vec<String> = flat
        .values()
        .map(|shown| match shown {
            Shown::Text(text) | Shown::Other(text) => text.clone(),
        })
        .collect();
    (!flat.is_empty()).then(|| texts.join(", "))
}

fn add_table(flat: &mut Flat, prefix: &str, table: &Table) {
    for (key, value) in table.pairs::<Value, Value>().flatten() {
        let name = match &key {
            Value::String(s) => s.to_string_lossy(),
            Value::Integer(i) => i.to_string(),
            other => format!("{other:?}"),
        };
        let name = match prefix {
            "" => name,
            _ => format!("{prefix}.{name}"),
        };
        match value {
            Value::Table(inner) => add_table(flat, &name, &inner),
            Value::String(s) => {
                flat.insert(name, Shown::Text(s.to_string_lossy()));
            }
            Value::Function(_) => {
                flat.insert(name, Shown::Other("function".to_string()));
            }
            Value::Boolean(flag) => {
                flat.insert(name, Shown::Other(flag.to_string()));
            }
            Value::Integer(number) => {
                flat.insert(name, Shown::Other(number.to_string()));
            }
            other => {
                flat.insert(name, Shown::Other(format!("{other:?}")));
            }
        }
    }
}
