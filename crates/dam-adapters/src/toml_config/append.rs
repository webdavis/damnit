use std::path::Path as FsPath;

use toml::Value;

use super::rewrite::{edit, ready_for_a_new_table};
use super::{ConfigError, parse_config};

pub(super) fn append_remote(
    path: &FsPath,
    name: &str,
    url: &str,
) -> Result<Option<String>, ConfigError> {
    edit(path, |text| {
        let mut text = ready_for_a_new_table(text);
        text.push_str(&format!(
            "[remote.{name}]\nurl = {}\n",
            Value::String(url.to_string())
        ));
        readable_with_the_new_remote(&text, name)?;
        Ok(text)
    })
}

fn readable_with_the_new_remote(text: &str, name: &str) -> Result<(), ConfigError> {
    parse_config(text)?.remote(name).map(|_| ()).ok_or_else(|| {
        ConfigError::Invalid(format!("the new config would not hold remote {name:?}"))
    })
}

#[cfg(test)]
mod tests;
