mod grant;
mod terms;

use dam_application::{
    Standing, TokenConfig, TokenFault, TokenWarning, Tokens, disable_shared_values,
};
use dam_domain::Token;
use mlua::Value;

use super::shape::text;
use super::strict::StrictMap;
use terms::read_terms;

const KNOWN_SCHEMA: i64 = 1;

pub(super) fn read_tokens(root: &StrictMap) -> Tokens {
    let mut warnings = Vec::new();
    let tokens = root.map("token").unwrap_or_else(|why| {
        warnings.push(TokenWarning::EveryTokenIgnored(why));
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
            let ignored = |why: TokenFault| TokenWarning::Ignored {
                name: name.to_string(),
                why,
            };
            let token = token
                .map_err(|why| warnings.push(ignored(why.into())))
                .ok()?;
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

fn newer_schema(schema: Option<&Value>) -> Option<TokenFault> {
    let found = match schema {
        None | Some(Value::Integer(KNOWN_SCHEMA)) => return None,
        Some(Value::Integer(n)) => {
            return Some(TokenFault::NewerSchema {
                found: *n,
                reads: KNOWN_SCHEMA,
            });
        }
        Some(Value::Number(n)) => format!("{n:?}"),
        Some(other) => {
            text(other).map_or_else(|| format!("a {}", other.type_name()), |s| format!("{s:?}"))
        }
    };
    Some(TokenFault::SchemaNotWhole { found })
}

fn readable_value(token: &StrictMap) -> Result<Token, TokenFault> {
    let value = token.get("value").ok_or(TokenFault::NoValue)?;
    let value = text(value).ok_or(TokenFault::ValueNotText)?;
    Token::parse(&value).map_err(|_| TokenFault::ValueNotAToken)
}
