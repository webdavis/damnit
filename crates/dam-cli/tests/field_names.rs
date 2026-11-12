mod support;

use serde_json::Value;
use support::help_flags::own_flags;
use support::sandbox::Sandbox;

fn shown(sb: &Sandbox, oid: &str) -> Value {
    let (ok, out, err) = sb.dam(&["show", oid, "--json"]);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

fn field<'a>(object: &'a Value, key: &str) -> Option<&'a Value> {
    [&object[key], &object["task"][key], &object["event"][key]]
        .into_iter()
        .find(|v| !v.is_null())
}

fn edit_flags(sb: &Sandbox) -> Vec<String> {
    own_flags(sb, &["edit"])
        .into_iter()
        .filter(|flag| flag != "editor")
        .collect()
}

fn key_of(flag: &str) -> &str {
    flag.strip_prefix("no_")
        .or_else(|| flag.strip_prefix("un"))
        .unwrap_or(flag)
}

fn flag_values(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items.iter().flat_map(flag_values).collect(),
        Value::Object(focus) => flag_values(&focus["day"]),
        other => vec![other.to_string()],
    }
}

fn every_field_set(sb: &Sandbox) -> (String, String) {
    let blocker = sb.new_object(&["blocker"]);
    let event = sb.new_object(&[
        "standup",
        "--kind",
        "event",
        "--start",
        "2030-01-07T09:00",
        "--end",
        "2030-01-07T09:30",
    ]);
    let task = sb.new_object(&["report"]);
    let task_flags = [
        "--subject",
        "write the report",
        "--body",
        "the body",
        "--priority",
        "2",
        "--due",
        "2030-01-07T15:00",
        "--deadline",
        "2030-01-08",
        "--labels",
        "work",
        "--labels",
        "deep",
        "--depends",
        &blocker,
        "--recurrence",
        "every week",
        "--attached",
        &event,
        "--estimate",
        "45",
        "--focus",
        "today",
    ];
    let (ok, _, err) = sb.dam(&[&["edit", &task][..], &task_flags].concat());
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["edit", &event, "--location", "room 4", "--labels", "work"]);
    assert!(ok, "{err}");
    (task, event)
}

#[test]
fn every_edit_flag_is_named_after_a_key_that_show_prints() {
    let _guard = support::guard("every_edit_flag_is_named_after_a_key_that_show_prints");
    let sb = Sandbox::new();
    let (task, event) = every_field_set(&sb);
    let (task, event) = (shown(&sb, &task), shown(&sb, &event));
    let (ok, toon, err) = sb.dam(&["show", task["oid"].as_str().unwrap(), "--toon"]);
    assert!(ok, "{err}");
    for flag in edit_flags(&sb) {
        let key = key_of(&flag);
        assert!(
            field(&task, key).or_else(|| field(&event, key)).is_some(),
            "--{} names no key in show --json",
            flag.replace('_', "-")
        );
        if field(&task, key).is_some() {
            assert!(
                toon.contains(&format!("{key}:")) || toon.contains(&format!("{key}[")),
                "--{flag} names no key in show --toon: {toon}"
            );
        }
    }
}

#[test]
fn a_value_read_from_show_passes_back_through_the_flag_of_the_same_name() {
    let _guard =
        support::guard("a_value_read_from_show_passes_back_through_the_flag_of_the_same_name");
    let sb = Sandbox::new();
    let (task, event) = every_field_set(&sb);
    let copies = [
        (task, sb.new_object(&["copy"])),
        (
            event,
            sb.new_object(&[
                "copy",
                "--kind",
                "event",
                "--start",
                "2031-01-01",
                "--end",
                "2031-01-02",
            ]),
        ),
    ];
    let keys: Vec<String> = edit_flags(&sb)
        .iter()
        .filter(|f| key_of(f) == f.as_str())
        .cloned()
        .collect();
    for (original, copy) in copies {
        let original = shown(&sb, &original);
        let mut line = vec!["edit".to_string(), copy.clone()];
        for key in keys.iter().filter(|k| field(&original, k).is_some()) {
            for value in flag_values(field(&original, key).unwrap()) {
                line.extend([format!("--{key}"), value]);
            }
        }
        let line: Vec<&str> = line.iter().map(String::as_str).collect();
        let (ok, _, err) = sb.dam(&line);
        assert!(ok, "{line:?}: {err}");
        let copied = shown(&sb, &copy);
        for key in keys.iter().filter(|k| field(&original, k).is_some()) {
            assert_eq!(
                field(&copied, key).map(flag_values),
                field(&original, key).map(flag_values),
                "--{key} did not carry the value show printed"
            );
        }
    }
}

#[test]
fn the_id_flag_of_new_names_the_key_that_new_and_show_print() {
    let _guard = support::guard("the_id_flag_of_new_names_the_key_that_new_and_show_print");
    let sb = Sandbox::new();
    assert!(own_flags(&sb, &["new"]).contains(&"id".to_string()));
    let (ok, out, err) = sb.dam(&["new", "report", "--id", "weekly", "--json"]);
    assert!(ok, "{err}");
    let made: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(made["id"], "weekly");
    let oid = made["oid"].as_str().unwrap();
    assert_eq!(shown(&sb, oid)["id"], "weekly");
    let (ok, toon, err) = sb.dam(&["show", oid, "--toon"]);
    assert!(ok, "{err}");
    assert!(toon.contains("id: weekly"), "{toon}");
}
