use super::key::{SettingKey, SettingKind};
use crate::ConfiguredDuration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingValue {
    Text(String),
    Flag(bool),
    List(Vec<String>),
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
            SettingKind::Flag => text.parse().map(SettingValue::Flag).map_err(|_| bad()),
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
        }
    }
}
