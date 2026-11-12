use std::path::Path as FsPath;

use toml_edit::DocumentMut;

use super::rewrite::edit;
use super::{ConfigError, parse_config};

pub(super) fn remove_token(path: &FsPath, name: &str) -> Result<Option<String>, ConfigError> {
    edit(path, |text| without_the_token(&text, name))
}

fn without_the_token(text: &str, name: &str) -> Result<String, ConfigError> {
    let mut document: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| ConfigError::Syntax(e.to_string()))?;
    let removed = document
        .get_mut("token")
        .and_then(|tokens| tokens.as_table_like_mut())
        .and_then(|tokens| tokens.remove(name));
    if removed.is_none() {
        return Err(ConfigError::Invalid(format!(
            "the config holds no token {name:?}"
        )));
    }
    let text = document.to_string();
    if parse_config(&text)?.tokens.names.iter().any(|n| n == name) {
        return Err(ConfigError::Invalid(format!(
            "the new config would still hold token {name:?}"
        )));
    }
    Ok(text)
}

#[cfg(test)]
mod tests;
