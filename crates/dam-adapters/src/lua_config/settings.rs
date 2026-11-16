use dam_application::{Config, ConfigError};
use dam_domain::{Categories, Category};

use super::filter::read_filter;
use super::hook::HOOK;
use super::remote::read_remote;
use super::strict::StrictMap;
use super::whole_number::whole_number;

pub(crate) const SETTINGS: [&str; 7] = [
    "owner",
    "done",
    "progress",
    "confirm_above",
    "remote",
    "category",
    "filter",
];
pub(crate) const DONE: [&str; 1] = ["interactive"];
pub(crate) const CATEGORY: [&str; 2] = ["values", "exclusive"];
const SECRETS: [&str; 2] = ["token", "schema"];

pub(super) fn read_settings(root: &StrictMap) -> Result<Config, ConfigError> {
    if let Some(key) = SECRETS.iter().find(|key| root.get(key).is_some()) {
        return Err(ConfigError::BelongsInSecrets {
            key: key.to_string(),
            at: root.place(),
        });
    }
    if let Some(hook) = root.get("hook") {
        return Err(hook_moved(root, hook));
    }
    root.only(&SETTINGS)?;
    let done_interactive = match root.map("done")? {
        Some(done) => {
            done.only(&DONE)?;
            done.bool("interactive")?
        }
        None => None,
    };
    let remotes = root
        .maps_in("remote")?
        .iter()
        .map(|(name, remote)| read_remote(name, remote))
        .collect::<Result<Vec<_>, _>>()?;
    let category_tables = root.maps_in("category")?;
    let categories = category_tables
        .iter()
        .map(|(name, category)| read_category(name, category))
        .collect::<Result<Vec<_>, _>>()?;
    let categories_writing_exclusive = category_tables
        .iter()
        .filter(|(_, category)| category.get("exclusive").is_some())
        .map(|(name, _)| name.clone())
        .collect();
    let categories = Categories::new(categories).map_err(ConfigError::BadCategories)?;
    let filters = root
        .maps_in("filter")?
        .iter()
        .map(|(name, filter)| read_filter(name, filter))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Config {
        owner: root.string("owner")?,
        done_interactive,
        progress: root.bool("progress")?,
        confirm_above: whole_number(root, "confirm_above")?,
        remotes,
        categories,
        categories_writing_exclusive,
        filters,
        hooks: Vec::new(),
        task_pulled: None,
        tokens: Default::default(),
    })
}

fn hook_moved(root: &StrictMap, hook: &mlua::Value) -> ConfigError {
    let held = hook
        .as_table()
        .map(|table| {
            table
                .pairs::<String, mlua::Value>()
                .filter_map(Result::ok)
                .map(|(k, _)| k)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ConfigError::HookMoved {
        config: root.place().file,
        events: HOOK
            .into_iter()
            .filter(|event| held.iter().any(|k| k == event))
            .map(String::from)
            .collect(),
    }
}

fn read_category(name: &str, category: &StrictMap) -> Result<Category, ConfigError> {
    category.only(&CATEGORY)?;
    let values = category
        .strings("values")?
        .ok_or_else(|| category.missing("values"))?;
    Ok(Category {
        name: name.to_string(),
        values,
        exclusive: category.bool("exclusive")?.unwrap_or(false),
    })
}
