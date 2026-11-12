use std::collections::BTreeSet;

use dam_application::TokenTerms;
use dam_domain::{Channel, Timestamp};
use dam_toml::StrictTable;
use toml::Value;

use super::grant::parse_grants;

const TOKEN_KEYS: [&str; 7] = [
    "value",
    "expires",
    "not_before",
    "via",
    "limits",
    "require",
    "grant",
];

pub(super) fn parse_terms(
    name: &str,
    t: &StrictTable,
    warnings: &mut Vec<String>,
) -> Result<TokenTerms, String> {
    t.only(&TOKEN_KEYS)?;
    no_limits(t)?;
    Ok(TokenTerms {
        expires: instant(t, "expires")?,
        not_before: instant(t, "not_before")?,
        via: channels(name, t, warnings)?,
        require: t
            .string_array("require", "an array of request details")?
            .unwrap_or_default(),
        grants: parse_grants(name, t, warnings)?,
    })
}

fn instant(t: &StrictTable, key: &str) -> Result<Option<Timestamp>, String> {
    let Some(value) = t.get(key) else {
        return Ok(None);
    };
    let parsed = match value {
        Value::Datetime(at) => at.to_string().parse::<Timestamp>().ok(),
        _ => None,
    };
    parsed.map(Some).ok_or_else(|| {
        t.refuse(
            key,
            format_args!("{key} must be a date-time with an offset"),
        )
    })
}

fn channels(
    name: &str,
    t: &StrictTable,
    warnings: &mut Vec<String>,
) -> Result<BTreeSet<Channel>, String> {
    let Some(words) = t.string_array("via", "an array of channel names")? else {
        return Ok([Channel::Local].into());
    };
    Ok(words
        .iter()
        .filter_map(|word| {
            let channel = Channel::parse(word);
            if channel.is_none() {
                warnings.push(format!("token {name}: unknown channel {word:?} is skipped"));
            }
            channel
        })
        .collect())
}

fn no_limits(t: &StrictTable) -> Result<(), String> {
    match t.get("limits") {
        None => Ok(()),
        Some(Value::Table(table)) if table.is_empty() => Ok(()),
        Some(Value::Table(_)) => Err("its limits are not enforced yet".to_string()),
        Some(_) => Err(t.refuse("limits", "limits must be a table of whole numbers")),
    }
}
