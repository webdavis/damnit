use dam_application::{Config, ConfigError, FilterConfig};
use dam_domain::{Categories, Category};

use super::remote::read_remote;
use super::strict::StrictMap;

const SETTINGS: [&str; 5] = ["owner", "done", "remote", "category", "filter"];
const SECRETS: [&str; 2] = ["token", "schema"];

pub(super) fn read_settings(root: &StrictMap) -> Result<Config, ConfigError> {
    if let Some(key) = SECRETS.iter().find(|key| root.get(key).is_some()) {
        return Err(ConfigError::Invalid(
            root.refuse(format_args!("{key} belongs in secrets.lua")),
        ));
    }
    root.only(&SETTINGS)?;
    let done_interactive = match root.map("done")? {
        Some(done) => {
            done.only(&["interactive"])?;
            done.bool("interactive")?.unwrap_or(false)
        }
        None => false,
    };
    let remotes = root
        .maps_in("remote")?
        .iter()
        .map(|(name, remote)| read_remote(name, remote))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = root
        .maps_in("category")?
        .iter()
        .map(|(name, category)| read_category(name, category))
        .collect::<Result<Vec<_>, _>>()?;
    let categories = Categories::new(categories)
        .map_err(|e| ConfigError::Invalid(format!("category: {e:?}")))?;
    let filters = root
        .maps_in("filter")?
        .iter()
        .map(|(name, filter)| read_filter(name, filter))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Config {
        owner: root.string("owner")?,
        done_interactive,
        remotes,
        categories,
        filters,
        tokens: Default::default(),
    })
}

fn read_filter(name: &str, filter: &StrictMap) -> Result<FilterConfig, ConfigError> {
    filter.only(&["query"])?;
    let query = filter
        .string("query")?
        .ok_or_else(|| ConfigError::Invalid(filter.refuse("query is missing")))?;
    Ok(FilterConfig {
        name: name.to_string(),
        query,
    })
}

fn read_category(name: &str, category: &StrictMap) -> Result<Category, ConfigError> {
    category.only(&["values", "exclusive"])?;
    let values = category
        .strings("values")?
        .ok_or_else(|| ConfigError::Invalid(category.refuse("values is missing")))?;
    Ok(Category {
        name: name.to_string(),
        values,
        exclusive: category.bool("exclusive")?.unwrap_or(false),
    })
}
