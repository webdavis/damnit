use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault};
use dam_domain::{Categories, parse_query};

use super::super::strict::StrictMap;
use super::rule::misshapen;

pub(super) fn listing(
    file: &Path,
    root: &StrictMap,
    categories: &Categories,
) -> Result<Option<String>, ConfigError> {
    let query = root
        .string("ls")
        .map_err(|e| misshapen(file, "ls")(e.into()))?;
    match &query {
        Some(text) if parse_query(text, categories).is_err() => Err(ConfigError::ProjectRule {
            file: file.to_path_buf(),
            key: "ls",
            fault: ProjectRuleFault::NotAQuery,
        }),
        _ => Ok(query),
    }
}
