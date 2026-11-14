use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum Value {
    Text(String),
    Flag(bool),
    List(Vec<String>),
    Unset,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum Setting {
    Leaf { value: Value, default: bool },
    Part(Part),
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

    pub(super) fn rows(&self, key: &str) -> Vec<(String, String)> {
        match self {
            Setting::Leaf { value, default } => {
                let marked = match (value, default) {
                    (Value::Unset, _) | (_, false) => "",
                    (_, true) => " (default)",
                };
                vec![(key.to_string(), format!("{}{marked}", value.text()))]
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
            Value::List(items) => items.join(", "),
            Value::Unset => "not set".to_string(),
        }
    }
}

fn joined(key: &str, name: &str) -> String {
    match key {
        "" => name.to_string(),
        _ => format!("{key}.{name}"),
    }
}
