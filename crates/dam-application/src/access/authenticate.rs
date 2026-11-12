use dam_domain::{Channel, Grants, Timestamp, Token, TokenError};

use super::Holder;
use crate::tokens::{Standing, TokenConfig, Tokens};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenRefusal {
    Malformed,
    UnknownVersion(String),
    Unknown,
    Disabled { name: String, why: String },
    NotYetValid { name: String },
    Expired { name: String },
    ChannelNotAllowed { name: String, channel: Channel },
}

impl TokenRefusal {
    pub fn name(&self) -> &'static str {
        match self {
            TokenRefusal::Malformed => "malformed_token",
            TokenRefusal::UnknownVersion(_) => "unknown_token_version",
            TokenRefusal::Unknown => "unknown_token",
            TokenRefusal::Disabled { .. } => "disabled_token",
            TokenRefusal::NotYetValid { .. } => "token_not_yet_valid",
            TokenRefusal::Expired { .. } => "expired_token",
            TokenRefusal::ChannelNotAllowed { .. } => "channel_not_allowed",
        }
    }
}

pub fn authenticate(
    tokens: &Tokens,
    presented: &str,
    channel: Channel,
    now: Timestamp,
) -> Result<Holder, TokenRefusal> {
    let presented = Token::parse(presented).map_err(|e| match e {
        TokenError::UnknownVersion(version) => TokenRefusal::UnknownVersion(version),
        TokenError::NotAToken | TokenError::Malformed | TokenError::Checksum => {
            TokenRefusal::Malformed
        }
    })?;
    let entry = matching_entry(tokens, &presented).ok_or(TokenRefusal::Unknown)?;
    let name = entry.name.clone();
    let terms = match &entry.standing {
        Standing::Usable(terms) => terms,
        Standing::Disabled(why) => {
            let why = why.clone();
            return Err(TokenRefusal::Disabled { name, why });
        }
    };
    if terms.not_before.is_some_and(|at| now < at) {
        return Err(TokenRefusal::NotYetValid { name });
    }
    if terms.expires.is_some_and(|at| now >= at) {
        return Err(TokenRefusal::Expired { name });
    }
    if !terms.via.contains(&channel) {
        return Err(TokenRefusal::ChannelNotAllowed { name, channel });
    }
    Ok(Holder {
        name,
        grants: Grants {
            holder: presented.key_id().clone(),
            grants: terms.grants.clone(),
        },
    })
}

fn matching_entry<'a>(tokens: &'a Tokens, presented: &Token) -> Option<&'a TokenConfig> {
    tokens
        .entries
        .iter()
        .fold(None, |found, entry| match presented.matches(&entry.value) {
            true => found.or(Some(entry)),
            false => found,
        })
}

#[cfg(test)]
mod tests;
