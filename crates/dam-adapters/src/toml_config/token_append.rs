use std::path::Path as FsPath;

use dam_application::NewToken;
use toml::Value;

use super::rewrite::{edit, ready_for_a_new_table};
use super::{ConfigError, parse_config};

pub(super) fn append_token(path: &FsPath, token: &NewToken) -> Result<Option<String>, ConfigError> {
    edit(path, |text| {
        let mut text = ready_for_a_new_table(text);
        text.push_str(&tables(token));
        readable_with_the_new_token(&text, token)?;
        Ok(text)
    })
}

fn tables(token: &NewToken) -> String {
    let name = token.name.as_str();
    let mut text = format!("[token.{name}]\n");
    line(
        &mut text,
        "value",
        Some(Value::String(token.value.expose())),
    );
    line(
        &mut text,
        "expires",
        token
            .expires
            .and_then(|at| at.to_string().parse().ok())
            .map(Value::Datetime),
    );
    line(
        &mut text,
        "via",
        token
            .via
            .as_ref()
            .map(|via| strings(via.iter().map(|c| c.as_str()))),
    );
    let grant = &token.grant;
    text.push_str(&format!("\n[[token.{name}.grant]]\n"));
    line(
        &mut text,
        "allow",
        Some(strings(grant.allow.iter().map(|p| p.as_str()))),
    );
    line(
        &mut text,
        "paths",
        grant
            .paths
            .as_ref()
            .map(|paths| strings(paths.iter().map(|p| p.to_string()))),
    );
    line(&mut text, "labels", grant.labels.as_ref().map(strings));
    line(&mut text, "remotes", grant.remotes.as_ref().map(strings));
    text
}

fn line(text: &mut String, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        text.push_str(&format!("{key} = {value}\n"));
    }
}

fn strings<S: ToString>(items: impl IntoIterator<Item = S>) -> Value {
    Value::Array(
        items
            .into_iter()
            .map(|s| Value::String(s.to_string()))
            .collect(),
    )
}

fn readable_with_the_new_token(text: &str, token: &NewToken) -> Result<(), ConfigError> {
    let tokens = parse_config(text)?.tokens;
    let held = tokens
        .entries
        .iter()
        .any(|e| e.name == token.name.as_str() && e.value == token.value);
    match held {
        true => Ok(()),
        false => Err(ConfigError::Invalid(format!(
            "the new config would not hold token {:?}",
            token.name.as_str()
        ))),
    }
}

#[cfg(test)]
mod tests;
