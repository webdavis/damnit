mod support;

use serde_json::{Value, json};
use support::commands::arguments;
use support::errors::kind_of;
use support::help_flags::own_flags;
use support::request::by_request;
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

fn accepted() -> Vec<(&'static str, Value)> {
    vec![
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
    ]
}

fn refused() -> Vec<(&'static str, Value)> {
    vec![
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
    ]
}

#[test]
fn a_request_holding_the_flags_a_command_takes_is_accepted_and_any_other_is_refused() {
    let _guard = support::guard(
        "a_request_holding_the_flags_a_command_takes_is_accepted_and_any_other_is_refused",
    );
    for (command, request) in accepted() {
        let file = format!("{command}.request.schema.json");
        assert_eq!(
            schema_errors(&request, &file),
            Vec::<String>::new(),
            "{request}"
        );
    }
    for (command, request) in refused() {
        let file = format!("{command}.request.schema.json");
        assert!(
            !schema_errors(&request, &file).is_empty(),
            "{file} took {request}"
        );
    }
}

#[test]
fn dam_takes_by_request_what_each_request_schema_accepts_and_refuses_the_rest() {
    let _guard = support::guard(
        "dam_takes_by_request_what_each_request_schema_accepts_and_refuses_the_rest",
    );
    let sb = Sandbox::new();
    let mut taken: Vec<String> = Vec::new();
    for (command, request) in accepted() {
        let out = by_request(&sb, &command.split('-').collect::<Vec<_>>(), &request);
        let kind = out
            .status
            .success()
            .then(String::new)
            .unwrap_or_else(|| kind_of(&out));
        assert!(
            !["usage", "parse"].contains(&kind.as_str()),
            "{command} {request}: {out:?}"
        );
        taken.push(format!("{command}.request.schema.json"));
    }
    for (command, request) in refused() {
        let out = by_request(
            &Sandbox::new(),
            &command.split('-').collect::<Vec<_>>(),
            &request,
        );
        assert!(
            ["usage", "parse"].contains(&kind_of(&out).as_str()),
            "{command} {request}: {out:?}"
        );
    }
    let mut published: Vec<String> = std::fs::read_dir(schemas())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".request.schema.json"))
        .collect();
    published.sort();
    taken.sort();
    assert_eq!(taken, published);
}
