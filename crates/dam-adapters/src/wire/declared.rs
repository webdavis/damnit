use dam_application::{Declared, RemoteMutation};
use serde_json::Value;

use super::without_local_fields;

const SECTIONS_OF_A_WIRE_OBJECT: [&str; 2] = ["task", "event"];

pub fn declared_on_the_wire(mutation: &RemoteMutation) -> Result<Declared, serde_json::Error> {
    let object = match &mutation.object {
        Some(object) => serde_json::to_value(without_local_fields(object))?,
        None => Value::Null,
    };
    mutation
        .fields
        .iter()
        .map(|field| {
            let value = wire_value(&object, field.as_str());
            Ok((*field, serde_json::to_string(&value)?))
        })
        .collect()
}

fn wire_value(object: &Value, name: &str) -> Value {
    std::iter::once(object)
        .chain(SECTIONS_OF_A_WIRE_OBJECT.iter().map(|s| &object[*s]))
        .find_map(|section| section.get(name))
        .cloned()
        .unwrap_or(Value::Null)
}
