use std::fmt;

pub fn remote_id(kind: char, id: &str) -> String {
    format!("{kind}:{id}")
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

pub fn split_remote_id(text: &str) -> Result<(char, &str), RemoteIdError> {
    let (kind, id) = text.split_once(':').ok_or(RemoteIdError::NoKind)?;
    let mut chars = kind.chars();
    match (chars.next(), chars.next()) {
        (Some(k @ ('p' | 's' | 'i')), None) if could_be_issued_by_todoist(id) => Ok((k, id)),
        (Some('p' | 's' | 'i'), None) => Err(RemoteIdError::BadId),
        _ => Err(RemoteIdError::NoKind),
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
