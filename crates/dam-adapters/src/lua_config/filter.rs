use dam_application::{ConfigError, Expected, FilterConfig, SelectError};
use dam_domain::{Categories, Command, Filter, chain_to_itself, query_table, scope};
use mlua::Value;

use super::filter_table::read_table;
use super::strict::StrictMap;
use super::write::table_literal;

pub(crate) const FILTER: [&str; 2] = ["any", "match"];
const QUERY: &str = "query";

pub(super) fn read_filter(
    name: &str,
    filter: &StrictMap,
    categories: &Categories,
) -> Result<FilterConfig, ConfigError> {
    let key = format!("filter.{name}");
    let file = filter.place().file;
    if let Some(query) = filter.string(QUERY).ok().flatten() {
        let was = format!("{key}.{QUERY}");
        return Err(query_text(file, (was, key), &query, categories));
    }
    let runs_a_function = match filter.get("match") {
        None => false,
        Some(Value::Function(_)) => true,
        Some(_) => return Err(filter.wrong("match", Expected::Function).into()),
    };
    let table = read_table(filter, &FILTER)?;
    checked(&table, file, key)?;
    Ok(FilterConfig {
        name: name.to_string(),
        table,
        runs_a_function,
    })
}

pub(super) fn query_text(
    file: String,
    (was, key): (String, String),
    query: &str,
    categories: &Categories,
) -> ConfigError {
    ConfigError::QueryText {
        file,
        was,
        key,
        table: query_table(query, categories).map(|t| table_literal(&t)),
    }
}

pub(super) fn checked(
    table: &dam_domain::FilterTable,
    file: String,
    key: String,
) -> Result<Filter, ConfigError> {
    let refused = |error| ConfigError::FilterTable {
        file: file.clone(),
        key: key.clone(),
        error: Box::new(error),
    };
    let filter = Filter::read(table).map_err(|e| refused(SelectError::Value(e)))?;
    scope(&filter, Command::List).map_err(|e| refused(SelectError::Defaults(e)))?;
    Ok(filter)
}

pub(super) fn none_reaches_itself(filters: &[FilterConfig], file: &str) -> Result<(), ConfigError> {
    let reaches = |name: &str| -> Vec<String> {
        filters
            .iter()
            .find(|f| f.name == name)
            .and_then(|f| Filter::read(&f.table).ok())
            .map(|f| f.saved_filters().into_iter().map(String::from).collect())
            .unwrap_or_default()
    };
    match filters
        .iter()
        .find_map(|f| chain_to_itself(&f.name, reaches))
    {
        Some(chain) => Err(ConfigError::FilterReachesItself {
            file: file.to_string(),
            chain,
        }),
        None => Ok(()),
    }
}
