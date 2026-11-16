use dam_domain::{FieldType, select_fields};
use serde_json::{Map, Value, json};

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/list.request.schema.json"
));

fn value_shape(ty: FieldType) -> Value {
    match ty {
        FieldType::YesNo => json!({"$ref": "#/$defs/yesNo"}),
        FieldType::Number(..) => json!({"$ref": "#/$defs/numbers"}),
        _ => json!({"$ref": "#/$defs/texts"}),
    }
}

fn fields() -> Map<String, Value> {
    let mut fields: Map<String, Value> = select_fields()
        .iter()
        .map(|f| {
            let mut shape = value_shape(f.ty);
            shape["description"] = format!(
                "As --{}: {}",
                f.key.replace(['_', '.'], "-"),
                f.shapes().join(", ")
            )
            .into();
            (f.key.to_string(), shape)
        })
        .collect();
    fields.insert(
        "any".into(),
        json!({
            "description": "At least one of these tables must hold, as well as every other key. A table may hold several fields and its own any.",
            "type": "array",
            "items": {"$ref": "#/$defs/group"}
        }),
    );
    fields
}

fn expected() -> Value {
    let mut properties = fields();
    properties.insert(
        "sort".into(),
        json!({
            "description": "As --sort: FIELD or FIELD:desc, or a list of them; empty values last.",
            "anyOf": [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]
        }),
    );
    for (key, what) in [
        ("limit", "List at most this many rows."),
        ("offset", "Skip this many rows first."),
    ] {
        properties.insert(
            key.into(),
            json!({"description": what, "type": "integer", "minimum": 0}),
        );
    }
    properties.insert(
        "global".into(),
        json!({"description": "As --global.", "type": "boolean"}),
    );
    properties.insert(
        "project".into(),
        json!({"description": "As --project.", "type": "string"}),
    );
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://github.com/webdavis/damnit/docs/schemas/list.request.schema.json",
        "title": "dam list request",
        "description": "What dam list --request reads: each key is a flag of dam list, its value the text typed after the flag. A number or true/false may be bare, and a list means the flag repeated.",
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
        "$defs": {
            "group": {"type": "object", "additionalProperties": false, "properties": fields()},
            "text": {"type": ["string", "number", "boolean"]},
            "texts": {"anyOf": [{"$ref": "#/$defs/text"}, {"type": "array", "items": {"$ref": "#/$defs/text"}}]},
            "numbers": {"$ref": "#/$defs/texts"},
            "yesNo": {"anyOf": [{"type": "boolean"}, {"enum": ["true", "false", "any"]}]}
        }
    })
}

#[test]
fn the_list_request_schema_is_the_field_table_written_as_keys() {
    let published: Value = serde_json::from_str(SCHEMA).unwrap();
    let expected = expected();
    assert!(
        published == expected,
        "docs/schemas/list.request.schema.json differs from the field table; it should be:\n{}",
        serde_json::to_string_pretty(&expected).unwrap()
    );
}

#[test]
fn a_request_the_schema_accepts_is_one_dam_list_reads() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let request = json!({"done": true, "priority": [1, 2], "any": [{"labels": "a", "any": [{"due": "today"}]}], "sort": "due:desc", "limit": 5});
    assert!(validator.is_valid(&request));
    assert!(!validator.is_valid(&json!({"duee": "today"})));
    assert!(!validator.is_valid(&json!({"any": [{"sort": "due"}]})));
}
