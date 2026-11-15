use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::commands::token::grant_view::GrantView;

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum Value {
    Text(String),
    Flag(bool),
    List(Vec<String>),
    Grants(Vec<GrantView>),
    Unset,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum Setting {
    Leaf { value: Value, default: bool },
    Hidden(Credential),
    Part(Part),
}

#[derive(Clone, Copy)]
pub(super) enum Credential {
    Text,
    Function,
}

impl Serialize for Credential {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("hidden", &true)?;
        map.serialize_entry("function", &matches!(self, Credential::Function))?;
        map.end()
    }
}

pub(super) struct Part(pub(super) Vec<(String, Setting)>);

impl Serialize for Part {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().map(|(name, setting)| (name, setting)))
    }
}

impl Setting {
    pub(super) fn set(value: Value) -> Setting {
        Setting::Leaf {
            value,
            default: false,
        }
    }

    pub(super) fn by_default(value: Value) -> Setting {
        Setting::Leaf {
            value,
            default: true,
        }
    }

    pub(super) fn unset() -> Setting {
        Setting::by_default(Value::Unset)
    }

    pub(super) fn part(entries: Vec<(String, Setting)>) -> Setting {
        Setting::Part(Part(entries))
    }

    pub(super) fn readings(&self, key: &str) -> Vec<(Vec<&str>, &Setting)> {
        let Setting::Part(Part(entries)) = self else {
            return Vec::new();
        };
        entries
            .iter()
            .flat_map(|(name, setting)| {
                let inside = match key.strip_prefix(name.as_str()) {
                    Some("") => vec![(Vec::new(), setting)],
                    Some(rest) => rest
                        .strip_prefix('.')
                        .map_or_else(Vec::new, |rest| setting.readings(rest)),
                    None => Vec::new(),
                };
                inside.into_iter().map(move |(mut path, setting)| {
                    path.insert(0, name.as_str());
                    (path, setting)
                })
            })
            .collect()
    }

    pub(super) fn rows(&self, key: &str) -> Vec<(String, String)> {
        match self {
            Setting::Leaf { value, default } => {
                let marked = match (value, default) {
                    (Value::Unset, _) | (_, false) => "",
                    (_, true) => " (default)",
                };
                vec![(key.to_string(), format!("{}{marked}", value.text()))]
            }
            Setting::Hidden(Credential::Text) => vec![(key.to_string(), "hidden".to_string())],
            Setting::Hidden(Credential::Function) => {
                vec![(key.to_string(), "hidden (function)".to_string())]
            }
            Setting::Part(Part(entries)) => entries
                .iter()
                .flat_map(|(name, setting)| setting.rows(&joined(key, name)))
                .collect(),
        }
    }
}

impl Value {
    fn text(&self) -> String {
        match self {
            Value::Text(text) => text.clone(),
            Value::Flag(flag) => flag.to_string(),
            Value::List(items) => or_a_dash(items.join(", ")),
            Value::Grants(grants) => or_a_dash(
                grants
                    .iter()
                    .map(GrantView::text)
                    .collect::<Vec<_>>()
                    .join(" | "),
            ),
            Value::Unset => "not set".to_string(),
        }
    }
}

fn or_a_dash(text: String) -> String {
    match text.is_empty() {
        true => "-".to_string(),
        false => text,
    }
}

fn joined(key: &str, name: &str) -> String {
    match key {
        "" => name.to_string(),
        _ => format!("{key}.{name}"),
    }
}
