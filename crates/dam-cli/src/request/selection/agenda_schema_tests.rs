use serde_json::{Value, json};

use super::schema_tests::fields;

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/agenda.request.schema.json"
));

fn expected() -> Value {
    let mut properties = fields();
    for (key, what) in [
        ("from", "As --from: a day, instant or anchor."),
        (
            "to",
            "As --to: a day or period counts whole, an instant is not included.",
        ),
        ("project", "As --project."),
    ] {
        properties.insert(key.into(), json!({"description": what, "type": "string"}));
    }
    properties.insert(
        "max_age".into(),
        json!({
            "description": "As --max-age: REMOTE=DURATION, or a list of them.",
            "anyOf": [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]
        }),
    );
    properties.insert(
        "global".into(),
        json!({"description": "As --global.", "type": "boolean"}),
    );
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://github.com/webdavis/damnit/docs/schemas/agenda.request.schema.json",
        "title": "dam agenda request",
        "description": "What dam agenda --request reads: each key is a flag of dam agenda, its value the text typed after the flag. A number or true/false may be bare, and a list means the flag repeated.",
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
fn the_agenda_request_schema_is_the_field_table_and_the_window_written_as_keys() {
    let published: Value = serde_json::from_str(SCHEMA).unwrap();
    let expected = expected();
    assert!(
        published == expected,
        "docs/schemas/agenda.request.schema.json differs from the field table; it should be:\n{}",
        serde_json::to_string_pretty(&expected).unwrap()
    );
}

#[test]
fn an_agenda_request_the_schema_accepts_holds_no_order() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&json!({"labels": "work", "to": "today", "max_age": "gcal=15m"})));
    assert!(!validator.is_valid(&json!({"sort": "start"})));
}
