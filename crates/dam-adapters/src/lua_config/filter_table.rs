use dam_application::{Expected, Misread};
use dam_domain::{FilterTable, TableValue};
use mlua::Value;

use super::shape::{list, text};
use super::strict::StrictMap;

const ANY: &str = "any";

pub(super) fn read_table(map: &StrictMap, skipped: &[&str]) -> Result<FilterTable, Misread> {
    let mut table = FilterTable::default();
    for (key, given) in map.entries() {
        if key == ANY {
            table.any = map
                .maps_listed(ANY)?
                .unwrap_or_default()
                .iter()
                .map(|alternative| read_table(alternative, &[]))
                .collect::<Result<_, _>>()?;
            continue;
        }
        if skipped.contains(&key) {
            continue;
        }
        let value = read_value(given).ok_or_else(|| map.wrong(key, Expected::FilterValue))?;
        table.fields.insert(key.to_string(), value);
    }
    Ok(table)
}

fn read_value(given: &Value) -> Option<TableValue> {
    match given {
        Value::Table(_) => list(given)?
            .iter()
            .map(scalar)
            .collect::<Option<_>>()
            .map(TableValue::Many),
        given => scalar(given).map(TableValue::One),
    }
}

fn scalar(given: &Value) -> Option<String> {
    match given {
        Value::Integer(n) => Some(n.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Boolean(yes) => Some(yes.to_string()),
        given => text(given),
    }
}
