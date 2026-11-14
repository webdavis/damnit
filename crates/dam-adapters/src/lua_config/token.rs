mod grant;
mod terms;

use dam_application::{Standing, TokenConfig, Tokens, disable_shared_values};
use dam_domain::Token;
use mlua::Value;

use super::shape::text;
use super::strict::StrictMap;
use terms::read_terms;

const KNOWN_SCHEMA: i64 = 1;

pub(super) fn read_tokens(root: &StrictMap) -> Tokens {
    let mut warnings = Vec::new();
    let tokens = root.map("token").unwrap_or_else(|why| {
        warnings.push(format!("every token is ignored: {why}"));
        None
    });
    let names = tokens
        .iter()
        .flat_map(|tokens| tokens.entries())
        .map(|(name, _)| name.to_string())
        .collect();
    let newer_schema = newer_schema(root.get("schema"));
    let entries = tokens
        .iter()
        .flat_map(|tokens| {
            tokens
                .entries()
                .map(move |(name, value)| (name, tokens.child(name, value)))
        })
        .filter_map(|(name, token)| {
            let ignored = |why: String| format!("token {name} is ignored: {why}");
            let token = token.map_err(|why| warnings.push(ignored(why))).ok()?;
            let value = readable_value(&token)
                .map_err(|why| warnings.push(ignored(why)))
                .ok()?;
            let standing = match &newer_schema {
                Some(why) => Standing::Disabled(why.clone()),
                None => read_terms(name, &token, &mut warnings)
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
        names,
        entries: disable_shared_values(entries),
        warnings,
    }
}

fn newer_schema(schema: Option<&Value>) -> Option<String> {
    let found = match schema {
        None | Some(Value::Integer(KNOWN_SCHEMA)) => return None,
        Some(Value::Integer(n)) => {
            return Some(format!(
                "secrets.lua is schema {n}, and this dam reads schema {KNOWN_SCHEMA}"
            ));
        }
        Some(Value::Number(n)) => format!("{n:?}"),
        Some(other) => {
            text(other).map_or_else(|| format!("a {}", other.type_name()), |s| format!("{s:?}"))
        }
    };
    Some(format!(
        "schema in secrets.lua must be a whole number, and it is {found}"
    ))
}

fn readable_value(token: &StrictMap) -> Result<Token, String> {
    let value = token.get("value").ok_or("it has no value")?;
    let value = text(value).ok_or("its value must be a string")?;
    Token::parse(&value).map_err(|_| "its value is not a damnit token".to_string())
}
