use dam_application::RemoteMutation;
use dam_domain::Field;
use serde_json::Value;

use super::without_local_fields;

const ALWAYS_JSON: &str = "a wire object always serializes as JSON";

const SECTIONS_OF_A_WIRE_OBJECT: [&str; 2] = ["task", "event"];

pub fn declared_on_the_wire(mutation: &RemoteMutation) -> String {
    let object = mutation
        .object
        .as_ref()
        .map(|o| serde_json::to_value(without_local_fields(o)).expect(ALWAYS_JSON))
        .unwrap_or(Value::Null);
    let declared: Vec<(&str, Value)> = Field::ALL
        .into_iter()
        .filter(|f| mutation.fields.contains(f))
        .map(|f| (f.as_str(), wire_value(&object, f.as_str())))
        .collect();
    serde_json::to_string(&declared).expect(ALWAYS_JSON)
}

fn wire_value(object: &Value, name: &str) -> Value {
    std::iter::once(object)
        .chain(SECTIONS_OF_A_WIRE_OBJECT.iter().map(|s| &object[*s]))
        .find_map(|section| section.get(name))
        .cloned()
        .unwrap_or(Value::Null)
}
