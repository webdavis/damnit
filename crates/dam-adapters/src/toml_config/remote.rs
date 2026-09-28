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
    let stale = optional_duration_keeping_its_text(name, t, "stale")?.map(|d| d.value);
    let deadline = optional_duration_keeping_its_text(name, t, "deadline")?;
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
    let declared = names_declared_under_credentials(name, t)?;
    let mut candidates: Vec<(u8, CredentialSpec)> = Vec::new();
    for (key, value) in t {
        if SETTING_KEYS.contains(&key.as_str()) {
            continue;
        }
        if !is_credential_key(key, &declared) {
            return Err(unknown_key(name, key));
        }
        candidates.push((
            precedence_value_then_command_then_variable(key),
            parse_credential(name, key, value)?,
        ));
    }
    let credentials = one_spec_per_name_at_its_highest_precedence(candidates);
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

const SETTING_KEYS: [&str; 5] = ["url", "stale", "deadline", "path", "credentials"];

fn names_declared_under_credentials(remote: &str, t: &Table) -> Result<Vec<String>, ConfigError> {
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

fn precedence_value_then_command_then_variable(key: &str) -> u8 {
    if key.ends_with("_command") {
        1
    } else if key.ends_with("_env") {
        2
    } else {
        0
    }
}

fn one_spec_per_name_at_its_highest_precedence(
    mut candidates: Vec<(u8, CredentialSpec)>,
) -> Vec<CredentialSpec> {
    candidates.sort_by(|a, b| a.1.name().cmp(b.1.name()).then(a.0.cmp(&b.0)));
    candidates.dedup_by(|a, b| a.1.name() == b.1.name());
    candidates.into_iter().map(|(_, spec)| spec).collect()
}

fn is_credential_key(key: &str, declared: &[String]) -> bool {
    key.ends_with("_command") || key.ends_with("_env") || declared.iter().any(|d| d == key)
}

fn unknown_key(remote: &str, key: &str) -> ConfigError {
    ConfigError::Invalid(format!(
        "remote.{remote}: {key} is not a key dam knows. The settings are {}; a credential is \
         <name>_command, <name>_env, or <name> with <name> listed in credentials",
        SETTING_KEYS.join(", ")
    ))
}

fn optional_duration_keeping_its_text(
    remote: &str,
    t: &Table,
    key: &str,
) -> Result<Option<ConfiguredDuration>, ConfigError> {
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
