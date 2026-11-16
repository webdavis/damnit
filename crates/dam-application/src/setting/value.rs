use dam_domain::FilterTable;

use super::key::{SettingKey, SettingKind};
use crate::{ConfiguredDuration, RemoteSync};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingValue {
    Text(String),
    Flag(bool),
    List(Vec<String>),
    Count(usize),
    Table(FilterTable),
}

#[derive(Debug, PartialEq, Eq)]
pub struct BadValue {
    pub key: String,
    pub value: String,
    pub expected: SettingKind,
}

impl SettingKey {
    pub fn value(&self, text: &str) -> Result<SettingValue, BadValue> {
        let bad = || BadValue {
            key: self.dotted(),
            value: text.to_string(),
            expected: self.kind,
        };
        match self.kind {
            SettingKind::Text => Ok(SettingValue::Text(text.to_string())),
            SettingKind::Duration => ConfiguredDuration::parse(text)
                .map(|_| SettingValue::Text(text.to_string()))
                .ok_or_else(bad),
            SettingKind::Interval => ConfiguredDuration::parse(text)
                .filter(|d| d.value >= RemoteSync::FLOOR)
                .map(|_| SettingValue::Text(text.to_string()))
                .ok_or_else(bad),
            SettingKind::Flag => text.parse().map(SettingValue::Flag).map_err(|_| bad()),
            SettingKind::Count => text.parse().map(SettingValue::Count).map_err(|_| bad()),
            SettingKind::List => Ok(SettingValue::List(
                text.split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(str::to_string)
                    .collect(),
            )),
        }
    }
}

impl SettingValue {
    pub fn text(&self) -> String {
        match self {
            SettingValue::Text(text) => text.clone(),
            SettingValue::Flag(flag) => flag.to_string(),
            SettingValue::List(items) => items.join(", "),
            SettingValue::Count(count) => count.to_string(),
            SettingValue::Table(table) => table_text(table),
        }
    }
}

fn table_text(table: &FilterTable) -> String {
    let fields = table
        .fields
        .iter()
        .map(|(key, value)| format!("{key} = {}", value.texts().join(", ")));
    let any = table
        .any
        .iter()
        .map(|t| format!("any {{ {} }}", table_text(t)));
    fields.chain(any).collect::<Vec<_>>().join("; ")
}
