use dam_application::{
    ConfigError, ConfiguredDuration, Misread, RemoteConfig, RemoteName, RemoteSync,
};
use dam_domain::Path;

use super::strict::StrictMap;

pub(crate) const SETTINGS: [&str; 5] = ["url", "deadline", "sync", "path", "except"];
pub(crate) const SYNC: [&str; 2] = ["interval", "allowed"];

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
        deadline: duration(remote, "deadline")?,
        sync: sync(remote)?,
        path,
        except: except(remote)?,
    })
}

fn except(remote: &StrictMap) -> Result<Vec<Path>, Misread> {
    let mut at = remote.place();
    at.table = at.table.map(|table| format!("{table}.except"));
    let texts = remote.strings("except")?.unwrap_or_default();
    texts
        .into_iter()
        .map(|text| {
            Path::parse(&text).map_err(|why| Misread::NotAPath {
                text,
                at: at.clone(),
                why,
            })
        })
        .collect()
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

fn interval(sync: &StrictMap) -> Result<Option<ConfiguredDuration>, Misread> {
    match duration(sync, "interval")? {
        Some(interval) if interval.value < RemoteSync::FLOOR => Err(Misread::ShorterThanFloor {
            key: "interval".into(),
            text: interval.text,
            floor: RemoteSync::FLOOR_TEXT,
            at: sync.place(),
        }),
        interval => Ok(interval),
    }
}

fn sync(remote: &StrictMap) -> Result<RemoteSync, Misread> {
    let Some(sync) = remote.map("sync")? else {
        return Ok(RemoteSync::default());
    };
    sync.only(&SYNC)?;
    Ok(RemoteSync {
        interval: interval(&sync)?,
        allowed: sync.bool("allowed")?,
    })
}
