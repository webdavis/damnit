use std::collections::BTreeSet;

use dam_application::{Misread, TokenFault, TokenWarning};
use dam_domain::{Grant, Path, Permission, PermissionName};

use crate::lua_config::strict::StrictMap;

const GRANT_KEYS: [&str; 5] = ["allow", "paths", "labels", "own", "remotes"];

pub(super) fn read_grants(
    name: &str,
    token: &StrictMap,
    warnings: &mut Vec<TokenWarning>,
) -> Result<Vec<Grant>, TokenFault> {
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

fn read_grant(
    name: &str,
    grant: &StrictMap,
    warnings: &mut Vec<TokenWarning>,
) -> Result<Grant, TokenFault> {
    let names = grant
        .strings("allow")?
        .ok_or_else(|| grant.missing("allow"))?;
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
    warnings: &mut Vec<TokenWarning>,
) -> Result<BTreeSet<Permission>, TokenFault> {
    let mut allow = BTreeSet::new();
    for word in names {
        match PermissionName::classify(word) {
            PermissionName::Grantable(permission) => {
                allow.insert(permission);
            }
            PermissionName::Reserved => {
                return Err(TokenFault::Reserved(word.clone()));
            }
            PermissionName::Unknown => {
                warnings.push(TokenWarning::UnknownPermission {
                    name: name.to_string(),
                    word: word.clone(),
                });
            }
        }
    }
    Ok(allow)
}

fn paths(grant: &StrictMap) -> Result<Option<Vec<Path>>, Misread> {
    grant
        .strings("paths")?
        .map(|texts| {
            texts
                .iter()
                .map(|text| {
                    Path::parse(text).map_err(|why| Misread::NotAPath {
                        text: text.clone(),
                        at: grant.place(),
                        why,
                    })
                })
                .collect()
        })
        .transpose()
}
