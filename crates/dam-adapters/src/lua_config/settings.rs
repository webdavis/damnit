use dam_application::{Config, ConfigError};
use dam_domain::{Categories, Category};

use super::filter::read_filter;
use super::hook::read_hooks;
use super::remote::read_remote;
use super::strict::StrictMap;

pub(crate) const SETTINGS: [&str; 6] = ["owner", "done", "remote", "category", "filter", "hook"];
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
    let (hooks, task_pulled) = read_hooks(root)?;
    Ok(Config {
        owner: root.string("owner")?,
        done_interactive,
        remotes,
        categories,
        categories_writing_exclusive,
        filters,
        hooks,
        task_pulled,
        tokens: Default::default(),
    })
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
