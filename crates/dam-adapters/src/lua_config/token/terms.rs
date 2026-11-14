use std::collections::BTreeSet;

use dam_application::{Expected, Misread, TokenFault, TokenTerms, TokenWarning};
use dam_domain::{Channel, Timestamp};
use mlua::Value;

use super::grant::read_grants;
use crate::lua_config::shape::text;
use crate::lua_config::strict::StrictMap;

const TOKEN_KEYS: [&str; 7] = [
    "value",
    "expires",
    "not_before",
    "via",
    "limits",
    "require",
    "grant",
];

pub(super) fn read_terms(
    name: &str,
    token: &StrictMap,
    warnings: &mut Vec<TokenWarning>,
) -> Result<TokenTerms, TokenFault> {
    token.only(&TOKEN_KEYS)?;
    no_limits(token)?;
    Ok(TokenTerms {
        expires: instant(token, "expires")?,
        not_before: instant(token, "not_before")?,
        via: channels(name, token, warnings)?,
        require: token.strings("require")?.unwrap_or_default(),
        grants: read_grants(name, token, warnings)?,
    })
}

fn instant(token: &StrictMap, key: &str) -> Result<Option<Timestamp>, Misread> {
    let Some(value) = token.get(key) else {
        return Ok(None);
    };
    text(value)
        .and_then(|at| at.parse::<Timestamp>().ok())
        .map(Some)
        .ok_or_else(|| token.wrong(key, Expected::DateTime))
}

fn channels(
    name: &str,
    token: &StrictMap,
    warnings: &mut Vec<TokenWarning>,
) -> Result<BTreeSet<Channel>, Misread> {
    let Some(words) = token.strings("via")? else {
        return Ok([Channel::Local].into());
    };
    Ok(words
        .iter()
        .filter_map(|word| {
            let channel = Channel::parse(word);
            if channel.is_none() {
                warnings.push(TokenWarning::UnknownChannel {
                    name: name.to_string(),
                    word: word.clone(),
                });
            }
            channel
        })
        .collect())
}

fn no_limits(token: &StrictMap) -> Result<(), TokenFault> {
    match token.get("limits") {
        None => Ok(()),
        Some(Value::Table(table)) if table.is_empty() => Ok(()),
        Some(Value::Table(_)) => Err(TokenFault::LimitsNotEnforced),
        Some(_) => Err(token.wrong("limits", Expected::TableOfWholeNumbers).into()),
    }
}
