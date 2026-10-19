use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteId {
    Project(String),
    Section(String),
    Item(String),
}

impl fmt::Display for RemoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteId::Project(id) => write!(f, "p:{id}"),
            RemoteId::Section(id) => write!(f, "s:{id}"),
            RemoteId::Item(id) => write!(f, "i:{id}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteIdError {
    NoKind,
    BadId,
}

impl fmt::Display for RemoteIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteIdError::NoKind => f.write_str("a Todoist id starts with p:, s: or i:"),
            RemoteIdError::BadId => {
                f.write_str("a Todoist id holds only letters, digits, hyphens and underscores")
            }
        }
    }
}

impl FromStr for RemoteId {
    type Err = RemoteIdError;

    fn from_str(text: &str) -> Result<RemoteId, RemoteIdError> {
        let (kind, id) = text.split_once(':').ok_or(RemoteIdError::NoKind)?;
        let variant = match kind {
            "p" => RemoteId::Project,
            "s" => RemoteId::Section,
            "i" => RemoteId::Item,
            _ => return Err(RemoteIdError::NoKind),
        };
        match could_be_issued_by_todoist(id) {
            true => Ok(variant(id.to_string())),
            false => Err(RemoteIdError::BadId),
        }
    }
}

fn could_be_issued_by_todoist(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn api_priority(dam: u8) -> u8 {
    5u8.saturating_sub(dam.clamp(1, 4))
}

pub fn dam_priority(api: u8) -> u8 {
    5u8.saturating_sub(api.clamp(1, 4))
}
