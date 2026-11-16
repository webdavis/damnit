use dam_application::{Config, SettingsFile, UseCaseError};
use dam_domain::parse_query;

use super::entry::{entry_key, setting_key};
use super::filter_written::{FilterCreated, FilterRemoved};
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

pub(super) fn create(
    config: &Config,
    file: &dyn SettingsFile,
    name: &str,
    query: &str,
) -> Result<Report, CliError> {
    let key = setting_key("filter", name, "query")?;
    if config.filters.iter().any(|f| f.name == name) {
        return Err(CliError::Usage(format!(
            "filter {name} exists; change its query with dam config set filter.{name}.query '<query>'"
        )));
    }
    parse_query(query, &config.categories)
        .map_err(|e| CliError::UseCase(UseCaseError::Query(e)))?;
    file.set(
        &key,
        &dam_application::SettingValue::Text(query.to_string()),
    )?;
    report(&FilterCreated { name, query })
}

pub(super) fn remove(
    config: &Config,
    file: &dyn SettingsFile,
    name: &str,
) -> Result<Report, CliError> {
    let filter = config
        .filters
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| {
            CliError::Usage(format!(
                "there is no filter {name}; dam filter list shows the saved filters"
            ))
        })?;
    if filter.rule.runs_a_function() {
        return Err(CliError::Usage(format!(
            "filter {name} runs a match function, which is written by hand; \
             remove filter.{name} from config.lua by hand"
        )));
    }
    file.unset(&entry_key("filter", name))?;
    report(&FilterRemoved { name })
}
