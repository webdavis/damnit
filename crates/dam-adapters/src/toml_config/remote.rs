use dam_application::{ConfiguredDuration, CredentialSpec, RemoteConfig, RemoteName, Secret};
use dam_domain::Path;
use toml::{Table, Value};

use super::{ConfigError, parse_duration};

pub(super) fn parse_remote(name: &str, t: &Table) -> Result<RemoteConfig, ConfigError> {
    let url = t
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| ConfigError::Invalid(format!("remote.{name} needs a url")))?;
    let helper = url
        .split_once("::")
        .map(|(h, _)| h)
        .filter(|h| !h.is_empty())
        .ok_or_else(|| {
            ConfigError::Invalid(format!(
                "remote.{name}: url {url:?} must look like <helper>::"
            ))
        })?;
    let stale = duration(name, t, "stale")?.map(|d| d.value);
    let deadline = duration(name, t, "deadline")?;
    let path = t
        .get("path")
        .map(|v| {
            v.as_str()
                .ok_or_else(|| {
                    ConfigError::Invalid(format!("remote.{name}: path must be a string"))
                })
                .and_then(|p| {
                    Path::parse(p)
                        .map_err(|e| ConfigError::Invalid(format!("remote.{name}: path: {e:?}")))
                })
        })
        .transpose()?;
    let declared = declared_credentials(name, t)?;
    let mut credentials = Vec::new();
    for (key, value) in t {
        if BASE_KEYS.contains(&key.as_str()) {
            continue;
        }
        if !is_credential_key(key, &declared) {
            return Err(unknown_key(name, key));
        }
        credentials.push(parse_credential(name, key, value)?);
    }
    Ok(RemoteConfig {
        name: RemoteName(name.to_string()),
        helper: helper.to_string(),
        url: url.to_string(),
        credentials,
        stale,
        deadline,
        path,
    })
}

/// The keys a remote table holds that are settings rather than credentials.
const BASE_KEYS: [&str; 5] = ["url", "stale", "deadline", "path", "credentials"];

/// The names under `credentials`, which is what makes a bare `<name>` key a
/// credential rather than a typo: the helper declares its names at connect
/// time, long after the config is parsed, so the config declares them itself.
fn declared_credentials(remote: &str, t: &Table) -> Result<Vec<String>, ConfigError> {
    let invalid = |why: &str| ConfigError::Invalid(format!("remote.{remote}: credentials {why}"));
    let Some(value) = t.get("credentials") else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| invalid("must be an array of names"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| invalid("must be an array of strings"))
        })
        .collect()
}

fn is_credential_key(key: &str, declared: &[String]) -> bool {
    key.ends_with("_command") || key.ends_with("_env") || declared.iter().any(|d| d == key)
}

fn unknown_key(remote: &str, key: &str) -> ConfigError {
    ConfigError::Invalid(format!(
        "remote.{remote}: {key} is not a key dam knows. The settings are {}; a credential is \
         <name>_command, <name>_env, or <name> with <name> listed in credentials",
        BASE_KEYS.join(", ")
    ))
}

/// One optional `<number><s|m|h>` key off a remote table, keeping the text.
fn duration(remote: &str, t: &Table, key: &str) -> Result<Option<ConfiguredDuration>, ConfigError> {
    t.get(key)
        .map(|v| {
            v.as_str()
                .ok_or_else(|| {
                    ConfigError::Invalid(format!("remote.{remote}: {key} must be a string"))
                })
                .and_then(|text| {
                    parse_duration(key, text).map(|value| ConfiguredDuration {
                        value,
                        text: text.to_string(),
                    })
                })
        })
        .transpose()
}

fn parse_credential(remote: &str, key: &str, value: &Value) -> Result<CredentialSpec, ConfigError> {
    let invalid = |why: &str| ConfigError::Invalid(format!("remote.{remote}: {key} {why}"));
    if let Some(name) = key.strip_suffix("_command") {
        let argv = value
            .as_array()
            .ok_or_else(|| invalid("must be an array of words"))?;
        let argv = argv
            .iter()
            .map(|w| {
                w.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| invalid("must be an array of strings"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if argv.is_empty() {
            return Err(invalid("must name a command"));
        }
        return Ok(CredentialSpec::Command {
            name: name.to_string(),
            argv,
        });
    }
    if let Some(name) = key.strip_suffix("_env") {
        let var = value
            .as_str()
            .ok_or_else(|| invalid("must be a variable name"))?;
        return Ok(CredentialSpec::Env {
            name: name.to_string(),
            var: var.to_string(),
        });
    }
    let literal = value.as_str().ok_or_else(|| invalid("must be a string"))?;
    Ok(CredentialSpec::Literal {
        name: key.to_string(),
        value: Secret::from(literal),
    })
}
