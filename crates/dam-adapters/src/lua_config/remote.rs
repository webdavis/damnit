use dam_application::{ConfigError, ConfiguredDuration, RemoteConfig, RemoteName};
use dam_domain::Path;

use super::strict::StrictMap;

const SETTINGS: [&str; 4] = ["url", "stale", "deadline", "path"];

pub(super) fn read_remote(name: &str, remote: &StrictMap) -> Result<RemoteConfig, ConfigError> {
    remote.only(&SETTINGS).map_err(|why| {
        ConfigError::Invalid(format!(
            "{why}. A remote takes {}; its credentials belong in secrets.lua",
            SETTINGS.join(", ")
        ))
    })?;
    let remote_name = RemoteName::parse(name).map_err(|e| ConfigError::Invalid(e.to_string()))?;
    let url = remote
        .string("url")?
        .ok_or_else(|| ConfigError::Invalid(remote.refuse("url is missing")))?;
    let helper = url
        .split_once("::")
        .map(|(helper, _)| helper)
        .filter(|helper| names_a_program(helper))
        .ok_or_else(|| {
            ConfigError::Invalid(remote.refuse(format_args!(
                "url {url:?} must look like <helper>::, the helper being letters, digits, - and _"
            )))
        })?
        .to_string();
    let path = remote
        .string("path")?
        .map(|p| Path::parse(&p).map_err(|e| remote.refuse(format_args!("path {p:?}: {e:?}"))))
        .transpose()?;
    Ok(RemoteConfig {
        name: remote_name,
        helper,
        url,
        credentials: Vec::new(),
        stale: duration(remote, "stale")?.map(|d| d.value),
        deadline: duration(remote, "deadline")?,
        path,
    })
}

fn names_a_program(helper: &str) -> bool {
    let plain = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    !helper.is_empty() && helper.chars().all(plain)
}

fn duration(remote: &StrictMap, key: &str) -> Result<Option<ConfiguredDuration>, ConfigError> {
    remote
        .string(key)?
        .map(|text| {
            ConfiguredDuration::parse(&text).ok_or_else(|| {
                remote
                    .refuse(format_args!("{key} {text:?} must be <number><s|m|h>"))
                    .into()
            })
        })
        .transpose()
}
