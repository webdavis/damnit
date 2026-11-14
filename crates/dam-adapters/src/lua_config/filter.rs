use dam_application::{Expected, FilterConfig, FilterRule, Misread};
use mlua::Value;

use super::strict::StrictMap;

pub(crate) const FILTER: [&str; 2] = ["query", "match"];

pub(super) fn read_filter(name: &str, filter: &StrictMap) -> Result<FilterConfig, Misread> {
    filter.only(&FILTER)?;
    let query = filter.string("query")?;
    let function = match filter.get("match") {
        None => false,
        Some(Value::Function(_)) => true,
        Some(_) => return Err(filter.wrong("match", Expected::Function)),
    };
    let rule = match (query, function) {
        (Some(query), false) => FilterRule::Query(query),
        (Some(query), true) => FilterRule::QueryThenFunction(query),
        (None, true) => FilterRule::Function,
        (None, false) => return Err(filter.missing("query or match")),
    };
    Ok(FilterConfig {
        name: name.to_string(),
        rule,
    })
}
