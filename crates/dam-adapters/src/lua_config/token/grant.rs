use std::collections::BTreeSet;

use dam_domain::{Grant, Path, Permission, PermissionName};

use crate::lua_config::strict::StrictMap;

const GRANT_KEYS: [&str; 5] = ["allow", "paths", "labels", "own", "remotes"];

pub(super) fn read_grants(
    name: &str,
    token: &StrictMap,
    warnings: &mut Vec<String>,
) -> Result<Vec<Grant>, String> {
    token
        .maps_listed("grant")?
        .unwrap_or_default()
        .iter()
        .map(|grant| {
            grant.only(&GRANT_KEYS)?;
            read_grant(name, grant, warnings)
        })
        .collect()
}

fn read_grant(name: &str, grant: &StrictMap, warnings: &mut Vec<String>) -> Result<Grant, String> {
    let names = grant
        .strings("allow")?
        .ok_or_else(|| grant.refuse("allow is missing"))?;
    Ok(Grant {
        allow: permissions(name, &names, warnings)?,
        paths: paths(grant)?,
        labels: grant.strings("labels")?.map(BTreeSet::from_iter),
        own: grant.bool("own")?.unwrap_or(false),
        remotes: grant.strings("remotes")?,
    })
}

fn permissions(
    name: &str,
    names: &[String],
    warnings: &mut Vec<String>,
) -> Result<BTreeSet<Permission>, String> {
    let mut allow = BTreeSet::new();
    for word in names {
        match PermissionName::classify(word) {
            PermissionName::Grantable(permission) => {
                allow.insert(permission);
            }
            PermissionName::Reserved => {
                return Err(format!("{word} is reserved and cannot be granted"));
            }
            PermissionName::Unknown => {
                warnings.push(format!(
                    "token {name}: unknown permission {word:?} is skipped"
                ));
            }
        }
    }
    Ok(allow)
}

fn paths(grant: &StrictMap) -> Result<Option<Vec<Path>>, String> {
    grant
        .strings("paths")?
        .map(|texts| {
            texts
                .iter()
                .map(|p| {
                    Path::parse(p).map_err(|e| grant.refuse(format_args!("path {p:?}: {e:?}")))
                })
                .collect()
        })
        .transpose()
}
