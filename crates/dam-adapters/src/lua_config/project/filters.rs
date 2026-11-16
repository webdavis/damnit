use std::path::Path;

use dam_application::{ConfigError, FilterConfig, ProjectRuleFault};
use dam_domain::Categories;

use super::super::filter::{checked, none_reaches_itself, query_text};
use super::super::filter_table::read_table;
use super::super::strict::StrictMap;

pub(super) fn filters(
    file: &Path,
    root: &StrictMap,
    categories: &Categories,
) -> Result<Vec<FilterConfig>, ConfigError> {
    let shown = file.display().to_string();
    let refused = |fault| ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "filter",
        fault,
    };
    let named = root
        .maps_in("filter")
        .map_err(|_| refused(ProjectRuleFault::Shape))?;
    let filters = named
        .into_iter()
        .map(|(name, filter)| {
            if filter.get("match").is_some() {
                return Err(refused(ProjectRuleFault::FilterIsAFunction));
            }
            let key = format!("filter.{name}");
            if let Some(query) = filter.string("query").ok().flatten() {
                let was = format!("{key}.query");
                return Err(query_text(shown.clone(), (was, key), &query, categories));
            }
            let table = read_table(&filter, &[]).map_err(|_| refused(ProjectRuleFault::Shape))?;
            checked(&table, shown.clone(), key)?;
            Ok(FilterConfig {
                name,
                table,
                runs_a_function: false,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    none_reaches_itself(&filters, &shown)?;
    Ok(filters)
}
