use mlua::{Table, Value};
use serde_json::{Map, Number};

use crate::lua_config::evaluate::described;

pub(in crate::lua_config) fn json_value(
    name: &str,
    value: &Value,
) -> Result<serde_json::Value, String> {
    Ok(match value {
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::Value::from(*i),
        Value::Number(n) => Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .ok_or_else(|| format!("{name} is {n}, which no request takes"))?,
        Value::String(s) => serde_json::Value::String(
            s.to_str()
                .map_err(|_| format!("{name} is not UTF-8 text"))?
                .to_string(),
        ),
        Value::Table(table) => json_table(name, table)?,
        other => {
            return Err(format!(
                "{name} is {}, which no request takes",
                described(other)
            ));
        }
    })
}

fn json_table(name: &str, table: &Table) -> Result<serde_json::Value, String> {
    let length = table.raw_len();
    let entries: Vec<(Value, Value)> = table
        .pairs()
        .collect::<mlua::Result<_>>()
        .map_err(|e| e.to_string())?;
    if length > 0 && entries.len() == length {
        let items = (1..=length)
            .map(|i| {
                let item: Value = table.raw_get(i).map_err(|e| e.to_string())?;
                json_value(&format!("{name}[{i}]"), &item)
            })
            .collect::<Result<_, _>>()?;
        return Ok(serde_json::Value::Array(items));
    }
    let mut fields = Map::new();
    for (key, value) in entries {
        let Value::String(key) = key else {
            return Err(format!("{name} has a key that is not a name"));
        };
        let key = key
            .to_str()
            .map_err(|_| format!("{name} has a key that is not UTF-8"))?
            .to_string();
        fields.insert(key.clone(), json_value(&key, &value)?);
    }
    Ok(serde_json::Value::Object(fields))
}
