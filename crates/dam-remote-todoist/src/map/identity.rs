//! How a Todoist object is named in dam: a `kind:id` pair, checked before it
//! is used to address anything upstream, and the priority scale both ways.

use std::fmt;

pub fn remote_id(kind: char, id: &str) -> String {
    format!("{kind}:{id}")
}

/// Why a stored `kind:id` pair cannot be used to address a Todoist object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteIdError {
    /// No `p:`, `s:` or `i:` prefix.
    NoKind,
    /// The id half is not one Todoist could have issued.
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

/// Splits a stored `kind:id` pair. The id half came from Todoist and goes back
/// to Todoist, so it is checked against the character set Todoist issues
/// (alphanumeric ids and hyphenated UUIDs) before it is used to address
/// anything.
pub fn split_remote_id(text: &str) -> Result<(char, &str), RemoteIdError> {
    let (kind, id) = text.split_once(':').ok_or(RemoteIdError::NoKind)?;
    let mut chars = kind.chars();
    match (chars.next(), chars.next()) {
        (Some(k @ ('p' | 's' | 'i')), None) => {
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(RemoteIdError::BadId);
            }
            Ok((k, id))
        }
        _ => Err(RemoteIdError::NoKind),
    }
}

pub fn api_priority(dam: u8) -> u8 {
    5u8.saturating_sub(dam.clamp(1, 4))
}

pub fn dam_priority(api: u8) -> u8 {
    5u8.saturating_sub(api.clamp(1, 4))
}
