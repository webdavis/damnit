use std::collections::BTreeSet;

use dam_domain::{Grant, Path, Permission, PermissionName};
use dam_toml::{Place, StrictTable};
use toml::Value;

const GRANT_KEYS: [&str; 5] = ["allow", "paths", "labels", "own", "remotes"];

pub(super) fn parse_grants(
    name: &str,
    t: &StrictTable,
    warnings: &mut Vec<String>,
) -> Result<Vec<Grant>, String> {
    let place = || Place::Section(format!("token.{name}.grant"));
    let tables = match t.get("grant") {
        None => return Ok(Vec::new()),
        Some(Value::Array(items)) => items,
        Some(_) => return Err(t.refuse("grant", "grant must be an array of tables")),
    };
    tables
        .iter()
        .map(|item| {
            let table = item
                .as_table()
                .ok_or_else(|| t.refuse("grant", "grant must be an array of tables"))?;
            let grant = StrictTable::new("", table, place(), &GRANT_KEYS)?;
            parse_grant(name, &grant, warnings)
        })
        .collect()
}

fn parse_grant(name: &str, t: &StrictTable, warnings: &mut Vec<String>) -> Result<Grant, String> {
    let names = t
        .string_array("allow", "an array of permission names")?
        .ok_or_else(|| t.refuse("allow", "allow is missing"))?;
    Ok(Grant {
        allow: permissions(name, &names, warnings)?,
        paths: paths(t)?,
        labels: t
            .string_array("labels", "an array of labels")?
            .map(BTreeSet::from_iter),
        own: t.bool("own")?.unwrap_or(false),
        remotes: t.string_array("remotes", "an array of remote names")?,
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

fn paths(t: &StrictTable) -> Result<Option<Vec<Path>>, String> {
    t.string_array("paths", "an array of paths")?
        .map(|texts| {
            texts
                .iter()
                .map(|p| {
                    Path::parse(p).map_err(|e| t.refuse("paths", format_args!("path {p:?}: {e:?}")))
                })
                .collect()
        })
        .transpose()
}
