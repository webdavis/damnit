use std::path::{Path as FsPath, PathBuf};

use dam_application::{Config, ConfigError, FilterConfig};
use dam_domain::{Categories, Category};

mod append;
mod file;
mod remote;
mod strict_table;
use append::append_remote;
pub use file::TomlConfigFile;
use remote::parse_remote;
pub(crate) use strict_table::{Place, StrictTable, parse_toml};

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn default_config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    base.join("dam").join("config.toml")
}

pub fn default_store_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local").join("share"));
    base.join("dam").join("dam.db")
}

fn load_config(path: &FsPath) -> Result<Config, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_config(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => parse_config(""),
        Err(e) => Err(ConfigError::Io(e.to_string())),
    }
}

const SECTIONS: [&str; 4] = ["done", "remote", "category", "filter"];

pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let table = parse_toml(text).map_err(ConfigError::Syntax)?;
    let root = StrictTable::new(text, &table, Place::TopLevel, &SECTIONS)?;
    let done_interactive = match root.section("done", &["interactive"])? {
        Some(done) => done.bool("interactive")?.unwrap_or(false),
        None => false,
    };
    let remotes = root
        .tables_in("remote")?
        .into_iter()
        .map(|(name, t)| parse_remote(name, &t))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = root
        .tables_in("category")?
        .into_iter()
        .map(|(name, t)| parse_category(name, &t))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = Categories::new(categories)
        .map_err(|e| ConfigError::Invalid(format!("category: {e:?}")))?;
    let filters = root
        .tables_in("filter")?
        .into_iter()
        .map(|(name, t)| parse_filter(name, &t))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Config {
        done_interactive,
        remotes,
        categories,
        filters,
    })
}

fn parse_filter(name: &str, t: &StrictTable) -> Result<FilterConfig, ConfigError> {
    t.only(&["query"])?;
    let query = t
        .string("query")?
        .ok_or_else(|| ConfigError::Invalid(t.refuse("query", "query is missing")))?;
    Ok(FilterConfig {
        name: name.to_string(),
        query,
    })
}

fn parse_category(name: &str, t: &StrictTable) -> Result<Category, ConfigError> {
    t.only(&["values", "exclusive"])?;
    let values = t
        .string_array("values", "an array of strings")?
        .ok_or_else(|| ConfigError::Invalid(t.refuse("values", "values is missing")))?;
    let exclusive = t.bool("exclusive")?.unwrap_or(false);
    Ok(Category {
        name: name.to_string(),
        values,
        exclusive,
    })
}

#[cfg(test)]
mod tests;
