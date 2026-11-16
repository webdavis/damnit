use std::path::Path;

use dam_application::ConfigError;
use dam_domain::{Categories, FilterTable};

use super::super::filter::{checked, query_text};
use super::super::filter_table::read_table;
use super::super::strict::StrictMap;
use super::rule::misshapen;

pub(super) const LIST: &str = "list";
pub(super) const LS: &str = "ls";

pub(super) fn listing(
    file: &Path,
    root: &StrictMap,
    categories: &Categories,
) -> Result<Option<FilterTable>, ConfigError> {
    let shown = file.display().to_string();
    if let Some(old) = root.get(LS) {
        let text = super::super::shape::text(old).unwrap_or_default();
        let key = (LS.to_string(), LIST.to_string());
        return Err(query_text(shown, key, &text, categories));
    }
    let Some(map) = root
        .map(LIST)
        .map_err(|e| misshapen(file, LIST)(e.into()))?
    else {
        return Ok(None);
    };
    let table = read_table(&map, &[]).map_err(|e| misshapen(file, LIST)(e.into()))?;
    checked(&table, shown, LIST.to_string())?;
    Ok(Some(table))
}
