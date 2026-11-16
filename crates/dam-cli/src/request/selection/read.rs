use dam_domain::{FilterTable, TableValue, closest_field, select_field};
use serde_json::{Map, Value};

use crate::args::selection::ordering::OrderArgs;
use crate::error::CliError;

const ANY: &str = "any";
const ORDER_KEYS: [&str; 3] = ["sort", "limit", "offset"];

pub(super) fn table(fields: &Map<String, Value>, verb: &str) -> Result<FilterTable, CliError> {
    group(fields, verb, |key| ORDER_KEYS.contains(&key))
}

pub(super) fn refuse_order(fields: &Map<String, Value>, verb: &str) -> Result<(), CliError> {
    match ORDER_KEYS.iter().find(|key| fields.contains_key(**key)) {
        Some(key) => Err(refuse(&format!("{key} is not a key {verb} takes"))),
        None => Ok(()),
    }
}

pub(super) fn order(fields: &Map<String, Value>) -> Result<OrderArgs, CliError> {
    let sort = fields.get("sort").map(|v| value("sort", v)).transpose()?;
    Ok(OrderArgs {
        sort: sort.map_or_else(Vec::new, |v| v.texts().to_vec()),
        limit: count(fields, "limit")?,
        offset: count(fields, "offset")?,
    })
}

fn group(
    fields: &Map<String, Value>,
    verb: &str,
    skipped: impl Fn(&str) -> bool,
) -> Result<FilterTable, CliError> {
    let mut table = FilterTable::default();
    for (key, given) in fields.iter().filter(|(key, _)| !skipped(key)) {
        if key == ANY {
            table.any = groups(given, verb)?;
        } else if select_field(key).is_some() {
            table.fields.insert(key.clone(), value(key, given)?);
        } else {
            return Err(unknown(key, verb));
        }
    }
    Ok(table)
}

fn groups(given: &Value, verb: &str) -> Result<Vec<FilterTable>, CliError> {
    let refused = || {
        refuse(
            r#"any must be a list of tables, such as [{"labels": "urgent"}, {"due": "..today"}]"#,
        )
    };
    let Value::Array(items) = given else {
        return Err(refused());
    };
    items
        .iter()
        .map(|item| match item {
            Value::Object(fields) => group(fields, verb, |_| false),
            _ => Err(refused()),
        })
        .collect()
}

fn value(key: &str, given: &Value) -> Result<TableValue, CliError> {
    match given {
        Value::Array(items) => items
            .iter()
            .map(|item| scalar(key, item))
            .collect::<Result<_, _>>()
            .map(TableValue::Many),
        given => scalar(key, given).map(TableValue::One),
    }
}

fn scalar(key: &str, given: &Value) -> Result<String, CliError> {
    match given {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(yes) => Ok(yes.to_string()),
        _ => Err(refuse(&format!(
            "{key} takes text, a number, true or false, or a list of them"
        ))),
    }
}

fn count(fields: &Map<String, Value>, key: &str) -> Result<Option<usize>, CliError> {
    fields
        .get(key)
        .map(|given| {
            given
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| refuse(&format!("{key} must be a whole number")))
        })
        .transpose()
}

fn unknown(key: &str, verb: &str) -> CliError {
    let hint = match closest_field(key) {
        Some(closest) => format!("the closest is {closest}"),
        None => format!("{verb} --help shows the fields"),
    };
    refuse(&format!("{key} is not a key {verb} takes; {hint}"))
}

fn refuse(why: &str) -> CliError {
    CliError::Usage(format!("--request: {why}"))
}
