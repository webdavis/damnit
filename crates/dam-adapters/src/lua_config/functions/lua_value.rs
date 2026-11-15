use mlua::{Lua, Value};

pub(super) fn lua_value(lua: &Lua, json: &serde_json::Value) -> mlua::Result<Value> {
    Ok(match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(i) => Value::Integer(i),
            None => Value::Number(n.as_f64().unwrap_or_default()),
        },
        serde_json::Value::String(s) => Value::String(lua.create_string(s)?),
        serde_json::Value::Array(items) => Value::Table(
            lua.create_sequence_from(
                items
                    .iter()
                    .map(|item| lua_value(lua, item))
                    .collect::<mlua::Result<Vec<_>>>()?,
            )?,
        ),
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (key, value) in map {
                table.set(key.as_str(), lua_value(lua, value)?)?;
            }
            Value::Table(table)
        }
    })
}
