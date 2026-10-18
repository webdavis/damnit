use dam_application::{ConfiguredDuration, CredentialSpec, RemoteConfig, RemoteName, Secret};
use dam_domain::Path;
use toml::Value;

use super::{ConfigError, StrictTable, parse_duration};

pub(super) fn parse_remote(name: &str, t: &StrictTable) -> Result<RemoteConfig, ConfigError> {
    let remote_name = RemoteName::parse(name).map_err(|e| ConfigError::Invalid(e.to_string()))?;
    let url = t
        .string("url")?
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
    let stale = optional_duration_keeping_its_text(t, "stale")?.map(|d| d.value);
    let deadline = optional_duration_keeping_its_text(t, "deadline")?;
    let path = t
        .string("path")?
        .map(|p| {
            Path::parse(&p).map_err(|e| ConfigError::Invalid(format!("remote.{name}: path: {e:?}")))
        })
        .transpose()?;
    let declared = t
        .string_array("credentials", "an array of names")?
        .unwrap_or_default();
    let mut candidates: Vec<(u8, CredentialSpec)> = Vec::new();
    for (key, value) in t.keys() {
        if SETTING_KEYS.contains(&key) {
            continue;
        }
        if !is_credential_key(key, &declared) {
            return Err(unknown_key(t, key));
        }
        candidates.push((
            precedence_value_then_command_then_variable(key),
            parse_credential(t, key, value)?,
        ));
    }
    let credentials = one_spec_per_name_at_its_highest_precedence(candidates);
    Ok(RemoteConfig {
        name: remote_name,
        helper: helper.to_string(),
        url: url.to_string(),
        credentials,
        stale,
        deadline,
        path,
    })
}

const SETTING_KEYS: [&str; 5] = ["url", "stale", "deadline", "path", "credentials"];

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

fn unknown_key(t: &StrictTable, key: &str) -> ConfigError {
    ConfigError::Invalid(format!(
        "{}. The settings are {}; a credential is <name>_command, <name>_env, or <name> with \
         <name> listed in credentials",
        t.refuse(key, format_args!("unknown key {key}")),
        SETTING_KEYS.join(", ")
    ))
}

fn optional_duration_keeping_its_text(
    t: &StrictTable,
    key: &str,
) -> Result<Option<ConfiguredDuration>, ConfigError> {
    t.string(key)?
        .map(|text| parse_duration(key, &text).map(|value| ConfiguredDuration { value, text }))
        .transpose()
}

fn parse_credential(
    t: &StrictTable,
    key: &str,
    value: &Value,
) -> Result<CredentialSpec, ConfigError> {
    let invalid = |why: &str| ConfigError::Invalid(t.refuse(key, format_args!("{key} {why}")));
    if let Some(name) = key.strip_suffix("_command") {
        let argv = t
            .string_array(key, "an array of words")?
            .expect("the key came from this table");
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

#[cfg(test)]
mod tests;
