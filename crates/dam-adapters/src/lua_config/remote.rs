use dam_application::{ConfigError, ConfiguredDuration, Misread, RemoteConfig, RemoteName};
use dam_domain::Path;

use super::strict::StrictMap;

const SETTINGS: [&str; 4] = ["url", "stale", "deadline", "path"];

pub(super) fn read_remote(name: &str, remote: &StrictMap) -> Result<RemoteConfig, ConfigError> {
    remote.only(&SETTINGS).map_err(|misread| match misread {
        Misread::UnknownKey { key, at } => Misread::UnknownRemoteKey {
            key,
            at,
            takes: SETTINGS.map(String::from).to_vec(),
        },
        other => other,
    })?;
    let remote_name = RemoteName::parse(name).map_err(ConfigError::BadRemoteName)?;
    let url = remote.string("url")?.ok_or_else(|| remote.missing("url"))?;
    let helper = url
        .split_once("::")
        .map(|(helper, _)| helper)
        .filter(|helper| names_a_program(helper))
        .ok_or_else(|| Misread::NotAHelperUrl {
            url: url.clone(),
            at: remote.place(),
        })?
        .to_string();
    let path = remote
        .string("path")?
        .map(|text| {
            Path::parse(&text).map_err(|why| Misread::NotAPath {
                text,
                at: remote.place(),
                why,
            })
        })
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

fn duration(remote: &StrictMap, key: &str) -> Result<Option<ConfiguredDuration>, Misread> {
    remote
        .string(key)?
        .map(|text| {
            ConfiguredDuration::parse(&text).ok_or_else(|| Misread::NotADuration {
                key: key.to_string(),
                text,
                at: remote.place(),
            })
        })
        .transpose()
}
