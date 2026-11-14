use dam_application::NewToken;
use dam_domain::Grant;

use super::insert::Indent;
use super::lua_text::{list_literal, string_literal, table_key};

pub(in crate::lua_config) fn token_entry(token: &NewToken, indent: &Indent) -> String {
    let inner = indent.deeper();
    let mut fields = vec![("value", string_literal(&token.value.expose()))];
    if let Some(expires) = token.expires {
        fields.push(("expires", string_literal(&expires.to_string())));
    }
    if let Some(via) = &token.via {
        fields.push(("via", list_literal(via.iter().map(|c| c.as_str()))));
    }
    if !token.require.is_empty() {
        fields.push(("require", list_literal(&token.require)));
    }
    let grants = format!(
        "{{\n{}{},\n{}}}",
        inner.deeper().leading,
        grant(&token.grant),
        inner.leading
    );
    fields.push(("grant", grants));
    let body: String = fields
        .iter()
        .map(|(name, value)| format!("{}{name} = {value},\n", inner.leading))
        .collect();
    format!(
        "{} = {{\n{body}{}}}",
        table_key(token.name.as_str()),
        indent.leading
    )
}

fn grant(grant: &Grant) -> String {
    let mut limits = vec![format!(
        "allow = {}",
        list_literal(grant.allow.iter().map(|p| p.as_str()))
    )];
    if let Some(paths) = &grant.paths {
        limits.push(format!(
            "paths = {}",
            list_literal(paths.iter().map(|p| p.to_string()))
        ));
    }
    if let Some(labels) = &grant.labels {
        limits.push(format!("labels = {}", list_literal(labels)));
    }
    if grant.own {
        limits.push("own = true".to_string());
    }
    if let Some(remotes) = &grant.remotes {
        limits.push(format!("remotes = {}", list_literal(remotes)));
    }
    format!("{{ {} }}", limits.join(", "))
}
