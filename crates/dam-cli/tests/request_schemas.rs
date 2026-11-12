mod support;

use serde_json::json;
use support::commands::arguments;
use support::help_flags::own_flags;
use support::sandbox::Sandbox;
use support::schema::{schema_errors, schemas};

const TAKING_FLAGS: [&str; 6] = ["new", "edit", "done", "mv", "rm", "token create"];

fn details_flag(spoken: &str) -> Vec<String> {
    match spoken {
        "token create" => vec![],
        _ => vec!["meta".to_string()],
    }
}

#[test]
fn every_writing_command_publishes_a_request_schema_named_after_its_flags() {
    let _guard =
        support::guard("every_writing_command_publishes_a_request_schema_named_after_its_flags");
    let sb = Sandbox::new();
    for spoken in TAKING_FLAGS {
        let command: Vec<&str> = spoken.split(' ').collect();
        let file = format!("{}.request.schema.json", command.join("-"));
        let text = std::fs::read_to_string(schemas().join(&file))
            .unwrap_or_else(|_| panic!("dam {spoken} has no {file}"));
        let schema: serde_json::Value = serde_json::from_str(&text).unwrap();
        let mut published: Vec<String> = schema["properties"]
            .as_object()
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default();
        let mut taken = [
            arguments(&sb, &command),
            own_flags(&sb, &command),
            details_flag(spoken),
        ]
        .concat();
        published.sort();
        taken.sort();
        assert_eq!(
            published, taken,
            "{file} names other keys than dam {spoken} takes"
        );
        assert_eq!(schema["additionalProperties"], false, "{file}");
    }
}

#[test]
fn a_request_holding_the_flags_a_command_takes_is_accepted_and_any_other_is_refused() {
    let _guard = support::guard(
        "a_request_holding_the_flags_a_command_takes_is_accepted_and_any_other_is_refused",
    );
    let accepted = [
        (
            "new",
            json!({"subject": "standup", "kind": "event", "start": "today", "end": "tomorrow"}),
        ),
        (
            "edit",
            json!({"oid": "1111", "estimate": "2h", "focus": "today", "no_due": true}),
        ),
        (
            "done",
            json!({"oid": "1111", "force": true, "children": "into:later", "depends": "drop"}),
        ),
        ("mv", json!({"oid": "1111", "path": "work/"})),
        ("rm", json!({"oid": "1111"})),
        (
            "token-create",
            json!({"name": "vpt", "allow": ["task:read"], "via": ["mcp"]}),
        ),
        (
            "day-set",
            json!({"day": "2026-11-11", "decisions": ["rest"]}),
        ),
    ];
    let refused = [
        ("new", json!({"subject": "standup", "kind": "event"})),
        ("new", json!({"subject": "milk", "label": ["errand"]})),
        ("new", json!({"subject": "milk", "priority": 5})),
        (
            "edit",
            json!({"oid": "1111", "due": "today", "no_due": true}),
        ),
        ("edit", json!({"oid": "1111", "editor": true})),
        ("done", json!({"oid": "1111", "children": "up"})),
        ("mv", json!({"oid": "1111"})),
        ("token-create", json!({"name": "vpt", "allow": []})),
        (
            "token-create",
            json!({"name": "vpt", "allow": ["task:read"], "expires": "soon"}),
        ),
        ("day-set", json!({"plan": []})),
    ];
    for (command, request) in accepted {
        let file = format!("{command}.request.schema.json");
        assert_eq!(
            schema_errors(&request, &file),
            Vec::<String>::new(),
            "{request}"
        );
    }
    for (command, request) in refused {
        let file = format!("{command}.request.schema.json");
        assert!(
            !schema_errors(&request, &file).is_empty(),
            "{file} took {request}"
        );
    }
}
