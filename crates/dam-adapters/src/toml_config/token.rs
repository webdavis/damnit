mod grant;
mod terms;

use dam_application::{Standing, TokenConfig, Tokens};
use dam_domain::Token;
use dam_toml::StrictTable;
use toml::Value;

use terms::parse_terms;

const KNOWN_SCHEMA: i64 = 1;

pub(super) fn parse_tokens(root: &StrictTable) -> Tokens {
    let mut warnings = Vec::new();
    let tables = match root.tables_in("token") {
        Ok(tables) => tables,
        Err(why) => {
            warnings.push(format!("every token is ignored: {why}"));
            Vec::new()
        }
    };
    let newer_schema = newer_schema(root.get("schema"));
    let entries: Vec<TokenConfig> = tables
        .into_iter()
        .filter_map(|(name, t)| {
            let value = readable_value(name, &t)
                .map_err(|w| warnings.push(w))
                .ok()?;
            let standing = match &newer_schema {
                Some(why) => Standing::Disabled(why.clone()),
                None => parse_terms(name, &t, &mut warnings)
                    .map_or_else(Standing::Disabled, Standing::Usable),
            };
            Some(TokenConfig {
                name: name.to_string(),
                value,
                standing,
            })
        })
        .collect();
    Tokens {
        entries: disable_shared_values(entries),
        warnings,
    }
}

fn disable_shared_values(mut entries: Vec<TokenConfig>) -> Vec<TokenConfig> {
    let sharers: Vec<Vec<String>> = entries
        .iter()
        .map(|entry| {
            entries
                .iter()
                .filter(|other| other.name != entry.name && other.value == entry.value)
                .map(|other| format!("token {}", other.name))
                .collect()
        })
        .collect();
    for (entry, others) in entries.iter_mut().zip(sharers) {
        if !others.is_empty() {
            let why = format!("it holds the same value as {}", others.join(", "));
            entry.standing = Standing::Disabled(why);
        }
    }
    entries
}

fn newer_schema(schema: Option<&Value>) -> Option<String> {
    match schema {
        None => None,
        Some(Value::Integer(KNOWN_SCHEMA)) => None,
        Some(other) => Some(format!(
            "the config is schema {other}, and this dam reads schema {KNOWN_SCHEMA}"
        )),
    }
}

fn readable_value(name: &str, t: &StrictTable) -> Result<Token, String> {
    let ignored = |why: &str| format!("token {name} is ignored: {why}");
    match t.get("value") {
        None => Err(ignored("it has no value")),
        Some(Value::String(text)) => {
            Token::parse(text).map_err(|_| ignored("its value is not a damnit token"))
        }
        Some(_) => Err(ignored("its value must be a string")),
    }
}

#[cfg(test)]
mod section_tests;
#[cfg(test)]
mod tests;
