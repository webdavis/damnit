use std::fmt;
use std::path::{Path as FsPath, PathBuf};
use std::time::Duration;

use dam_application::{Config, FilterConfig};
use dam_domain::{Categories, Category};

mod append;
mod remote;
mod strict_table;
pub use append::append_remote;
use remote::parse_remote;
pub(crate) use strict_table::{Place, StrictTable, parse_toml};

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Syntax(String),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(s) => write!(f, "reading config: {s}"),
            ConfigError::Syntax(s) => write!(f, "config is not TOML: {s}"),
            ConfigError::Invalid(s) => write!(f, "config: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<String> for ConfigError {
    fn from(refusal: String) -> ConfigError {
        ConfigError::Invalid(refusal)
    }
}

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

pub fn load_config(path: &FsPath) -> Result<Config, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_config(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => parse_config(""),
        Err(e) => Err(ConfigError::Io(e.to_string())),
    }
}

const SECTIONS: [&str; 4] = ["done", "remote", "category", "filter"];

pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let table = parse_toml(text).map_err(ConfigError::Syntax)?;
    let root = StrictTable::new(&table, Place::TopLevel, &SECTIONS)?;
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
        .ok_or_else(|| ConfigError::Invalid(format!("filter.{name} needs a query string")))?;
    Ok(FilterConfig {
        name: name.to_string(),
        query,
    })
}

fn parse_category(name: &str, t: &StrictTable) -> Result<Category, ConfigError> {
    t.only(&["values", "exclusive"])?;
    let values = t
        .string_array("values", "an array of strings")?
        .ok_or_else(|| ConfigError::Invalid(format!("category.{name} needs values")))?;
    let exclusive = t.bool("exclusive")?.unwrap_or(false);
    Ok(Category {
        name: name.to_string(),
        values,
        exclusive,
    })
}

pub fn parse_duration(field: &str, text: &str) -> Result<Duration, ConfigError> {
    let bad = || ConfigError::Invalid(format!("{field} {text:?}: expected <number><s|m|h>"));
    let mut chars = text.chars();
    let unit = chars.next_back().ok_or_else(bad)?;
    let n: u64 = chars.as_str().parse().map_err(|_| bad())?;
    let secs = match unit {
        's' => Some(n),
        'm' => n.checked_mul(60),
        'h' => n.checked_mul(3600),
        _ => return Err(bad()),
    }
    .ok_or_else(bad)?;
    Ok(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests;
