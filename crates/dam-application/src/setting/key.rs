#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKind {
    Text,
    Duration,
    Flag,
    List,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyRefusal {
    Unknown(String),
    Secret(String),
    Function(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingKey {
    pub path: Vec<String>,
    pub kind: SettingKind,
}

impl SettingKey {
    pub fn parse(key: &str) -> Result<SettingKey, KeyRefusal> {
        let (group, rest) = key.split_once('.').unwrap_or((key, ""));
        let named = rest.rsplit_once('.').filter(|(name, _)| !name.is_empty());
        let kind = match (group, rest, named.map(|(_, field)| field)) {
            ("owner", "", _) => Ok(SettingKind::Text),
            ("done", "interactive", _) | ("progress", "", _) => Ok(SettingKind::Flag),
            ("token", _, _) => Err(KeyRefusal::Secret(key.to_string())),
            ("hook", _, _) | ("filter", _, Some("match")) => {
                Err(KeyRefusal::Function(key.to_string()))
            }
            ("remote", _, Some("url" | "path")) | ("filter", _, Some("query")) => {
                Ok(SettingKind::Text)
            }
            ("remote", _, Some("deadline")) => Ok(SettingKind::Duration),
            ("remote", _, Some(_)) => Err(KeyRefusal::Secret(key.to_string())),
            ("category", _, Some("values")) => Ok(SettingKind::List),
            ("category", _, Some("exclusive")) => Ok(SettingKind::Flag),
            _ => Err(KeyRefusal::Unknown(key.to_string())),
        }?;
        let path = match named {
            Some((name, field)) => vec![group, name, field],
            _ if rest.is_empty() => vec![group],
            _ => vec![group, rest],
        };
        Ok(SettingKey {
            path: path.into_iter().map(str::to_string).collect(),
            kind,
        })
    }

    pub fn dotted(&self) -> String {
        self.path.join(".")
    }
}
