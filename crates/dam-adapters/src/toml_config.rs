use std::fmt;
use std::fs::{DirBuilder, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path as FsPath, PathBuf};
use std::time::Duration;

use dam_application::{Config, FilterConfig};
use dam_domain::{Categories, Category};
use toml::Table;
use toml::Value;

mod remote;
use remote::parse_remote;

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

pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let root: Table = text
        .parse()
        .map_err(|e: toml::de::Error| syntax_error(text, &e))?;
    let done_interactive = root
        .get("done")
        .and_then(|d| d.get("interactive"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let remotes = named_tables(&root, "remote")?
        .into_iter()
        .map(|(name, t)| parse_remote(name, t))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = named_tables(&root, "category")?
        .into_iter()
        .map(|(name, t)| parse_category(name, t))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = Categories::new(categories)
        .map_err(|e| ConfigError::Invalid(format!("category: {e:?}")))?;
    let filters = named_tables(&root, "filter")?
        .into_iter()
        .map(|(name, t)| {
            let query = t.get("query").and_then(Value::as_str).ok_or_else(|| {
                ConfigError::Invalid(format!("filter.{name} needs a query string"))
            })?;
            Ok(FilterConfig {
                name: name.to_string(),
                query: query.to_string(),
            })
        })
        .collect::<Result<Vec<_>, ConfigError>>()?;
    Ok(Config {
        done_interactive,
        remotes,
        categories,
        filters,
    })
}

/// A syntax error as its position plus the parser's own sentence. The snippet
/// toml renders is dropped: it echoes the offending line, where a literal
/// credential can sit.
fn syntax_error(text: &str, e: &toml::de::Error) -> ConfigError {
    let Some(span) = e.span() else {
        return ConfigError::Syntax(e.message().to_string());
    };
    let before = &text[..span.start.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    ConfigError::Syntax(format!("line {line}, column {column}: {}", e.message()))
}

fn named_tables<'a>(root: &'a Table, key: &str) -> Result<Vec<(&'a str, &'a Table)>, ConfigError> {
    let Some(section) = root.get(key) else {
        return Ok(Vec::new());
    };
    let table = section
        .as_table()
        .ok_or_else(|| ConfigError::Invalid(format!("[{key}] must hold named tables")))?;
    table
        .iter()
        .map(|(name, v)| {
            v.as_table()
                .map(|t| (name.as_str(), t))
                .ok_or_else(|| ConfigError::Invalid(format!("{key}.{name} must be a table")))
        })
        .collect()
}

fn parse_category(name: &str, t: &Table) -> Result<Category, ConfigError> {
    let values = t
        .get("values")
        .and_then(Value::as_array)
        .ok_or_else(|| ConfigError::Invalid(format!("category.{name} needs values")))?;
    let values = values
        .iter()
        .map(|v| {
            v.as_str().map(str::to_string).ok_or_else(|| {
                ConfigError::Invalid(format!("category.{name}: values must be strings"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let exclusive = t.get("exclusive").and_then(Value::as_bool).unwrap_or(false);
    Ok(Category {
        name: name.to_string(),
        values,
        exclusive,
    })
}

/// A `<number><s|m|h>` duration; `field` names the key in the refusal.
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

/// Adds one remote table. The `Some` answer is a warning for the operator,
/// returned rather than printed so the caller decides where it goes.
pub fn append_remote(path: &FsPath, name: &str, url: &str) -> Result<Option<String>, ConfigError> {
    let existing = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(ConfigError::Io(e.to_string())),
    };
    if parse_config(&existing)?.remote(name).is_some() {
        return Err(ConfigError::Invalid(format!(
            "remote {name} already exists"
        )));
    }
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&format!(
        "[remote.{name}]\nurl = {}\n",
        Value::String(url.to_string())
    ));
    let dir = parent_of(path);
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(io_error)?;
    write_private(path, &text)
}

fn parent_of(path: &FsPath) -> PathBuf {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn io_error(e: std::io::Error) -> ConfigError {
    ConfigError::Io(e.to_string())
}

/// Writes the config through a sibling temp file, so an interrupted write leaves
/// the old one intact, and leaves the result readable only by its owner: the
/// file is the documented home of a literal credential. The `Some` answer says
/// the file had been reachable by someone else.
fn write_private(path: &FsPath, text: &str) -> Result<Option<String>, ConfigError> {
    let widened = match std::fs::metadata(path) {
        Ok(meta) => meta.permissions().mode() & 0o777,
        Err(_) => 0,
    };
    let mut file = tempfile::NamedTempFile::new_in(parent_of(path)).map_err(io_error)?;
    file.as_file()
        .set_permissions(Permissions::from_mode(0o600))
        .map_err(io_error)?;
    file.write_all(text.as_bytes()).map_err(io_error)?;
    file.as_file().sync_all().map_err(io_error)?;
    file.persist(path).map_err(|e| io_error(e.error))?;
    if widened & 0o077 == 0 {
        return Ok(None);
    }
    Ok(Some(format!(
        "the config was mode {widened:o}; writing it 600, it can hold a credential"
    )))
}

#[cfg(test)]
mod tests;
