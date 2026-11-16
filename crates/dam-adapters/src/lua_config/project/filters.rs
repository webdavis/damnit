use std::path::Path;

use dam_application::{ConfigError, FilterConfig, FilterRule, ProjectRuleFault};
use dam_domain::{Categories, parse_query};

use super::super::strict::StrictMap;

pub(super) fn filters(file: &Path, root: &StrictMap) -> Result<Vec<FilterConfig>, ConfigError> {
    let refused = |fault| ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "filter",
        fault,
    };
    let named = root
        .maps_in("filter")
        .map_err(|_| refused(ProjectRuleFault::Shape))?;
    named
        .into_iter()
        .map(|(name, filter)| {
            if filter.get("match").is_some() {
                return Err(refused(ProjectRuleFault::FilterIsAFunction));
            }
            let query = filter
                .only(&["query"])
                .and_then(|_| filter.string("query"))
                .ok()
                .flatten()
                .ok_or_else(|| refused(ProjectRuleFault::Shape))?;
            if parse_query(&query, &Categories::default()).is_err() {
                return Err(refused(ProjectRuleFault::NotAQuery));
            }
            Ok(FilterConfig {
                name,
                rule: FilterRule::Query(query),
            })
        })
        .collect()
}
