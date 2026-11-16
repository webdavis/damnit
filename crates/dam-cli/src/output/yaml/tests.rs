use serde_json::{Value, json};
use yaml_rust2::{Yaml, YamlLoader};

use super::encode;

fn parsed(text: &str) -> Value {
    let documents = YamlLoader::load_from_str(text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    assert_eq!(documents.len(), 1, "{text}");
    as_json(&documents[0])
}

fn as_json(yaml: &Yaml) -> Value {
    match yaml {
        Yaml::Null => Value::Null,
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Integer(i) => json!(i),
        Yaml::Real(r) => json!(r.parse::<f64>().unwrap()),
        Yaml::String(s) => json!(s),
        Yaml::Array(items) => items.iter().map(as_json).collect(),
        Yaml::Hash(pairs) => pairs
            .iter()
            .map(|(k, v)| (k.as_str().expect("a string key").to_string(), as_json(v)))
            .collect::<serde_json::Map<_, _>>()
            .into(),
        other => panic!("not data: {other:?}"),
    }
}

fn round_trips(value: &Value) {
    let text = encode(value);
    assert_eq!(&parsed(&text), value, "{text}");
}

const LOOKALIKES: [&str; 34] = [
    "yes",
    "No",
    "TRUE",
    "false",
    "on",
    "OFF",
    "y",
    "N",
    "null",
    "Null",
    "~",
    "nan",
    ".inf",
    "-.inf",
    "12",
    "-3",
    "1.5",
    "1e3",
    "0x1F",
    "0o17",
    "2026-11-15",
    "12:30",
    "",
    " lead",
    "trail ",
    "a: b",
    "a #b",
    "- item",
    "? key",
    "&anchor",
    "*alias",
    "!tag",
    "@at",
    "`tick",
];

#[test]
fn a_string_yaml_would_read_as_another_type_reads_back_as_the_same_string() {
    for text in LOOKALIKES {
        round_trips(&json!(text));
        round_trips(&json!({ "key": text }));
        round_trips(&json!({ text: 1 }));
        round_trips(&json!([text]));
    }
}

#[test]
fn a_word_yaml_1_1_reads_as_a_boolean_or_null_is_quoted() {
    for word in ["yes", "No", "on", "OFF", "y", "N", "null", "~", "true"] {
        assert_eq!(encode(&json!(word)), format!("\"{word}\"\n"));
    }
}

#[test]
fn control_characters_and_line_breaks_read_back_unchanged() {
    for text in [
        "two\nlines",
        "tab\there",
        "cr\r",
        "nul\u{0}",
        "bell\u{7}",
        "del\u{7f}",
        "next\u{85}line",
        "sep\u{2028}\u{2029}",
        "bom\u{feff}",
        "quote\" and \\ slash",
        "emoji 🦀 and é",
    ] {
        round_trips(&json!(text));
        assert!(!encode(&json!(text)).trim_end().contains('\n'), "{text:?}");
    }
}

#[test]
fn a_plain_word_stays_plain() {
    assert_eq!(encode(&json!("buy oat milk")), "buy oat milk\n");
    assert_eq!(encode(&json!({"path": "work/home"})), "path: work/home\n");
}

#[test]
fn nested_maps_and_lists_read_back_unchanged() {
    round_trips(&json!({
        "objects": [
            {"oid": "3f3f3f3", "labels": ["errand", "yes"], "due": null, "done": false},
            {"oid": "abc", "labels": [], "meta": {}, "priority": 2, "estimate": 1.5},
            [[1, 2], [], {"a": {"b": [null]}}],
        ],
        "count": -4,
        "empty": "",
    }));
    round_trips(&json!([]));
    round_trips(&json!({}));
    round_trips(&json!(7));
    round_trips(&json!(true));
}

#[test]
fn a_map_reads_as_a_block() {
    assert_eq!(
        encode(&json!({"a": 1, "b": [true, {"c": null}], "d": {"e": "f"}})),
        "a: 1\nb:\n  - true\n  - c: null\nd:\n  e: f\n"
    );
}

#[test]
fn every_golden_json_answer_reads_back_unchanged_from_yaml() {
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut seen = 0;
    for entry in std::fs::read_dir(golden).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            round_trips(&serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap());
            seen += 1;
        }
    }
    assert!(seen > 20, "{seen}");
}
