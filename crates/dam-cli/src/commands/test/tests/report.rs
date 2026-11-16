use crate::args::PartName;

use super::{Files, Helpers, MARKER, attention_of, json, rows_of};

const REFUSED: &str = "did not sign in: fake answered 401: token <api_token> is not valid";

fn mixed() -> Files {
    let config = "return { remote = { fake = { url = \"fake::\" }, \
                  old = { url = \"old::\" }, work = { url = \"fake::\" } } }\n";
    let secrets = format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = \"tok\" }}, \
         old = {{ api_token = \"tok\" }}, work = {{ api_token = {MARKER:?} }} }} }}\n"
    );
    Files::new(config, &secrets)
}

#[test]
fn several_remotes_report_one_row_each_with_the_worst_on_the_part() {
    let report = mixed().report(&Helpers::default(), &[PartName::Credentials]);
    assert_eq!(
        rows_of(&report),
        [
            "  ✗ credentials: 3 remotes: 1 issue, 1 warning, 1 ok".to_string(),
            "    ✓ fake: signed in".to_string(),
            "    ⚠ old: dam-remote-old cannot test a login".to_string(),
            "      → update dam-remote-old".to_string(),
            format!("    ✗ work: {REFUSED}"),
            "      → check its credentials in secrets.lua".to_string(),
        ]
    );
    assert_eq!(
        attention_of(&report),
        [
            "  ✗ 1 issue to fix, 1 warning to look at".to_string(),
            format!("  1. credentials for work: {REFUSED}"),
            "  2. credentials for old: dam-remote-old cannot test a login".to_string(),
        ]
    );
}

#[test]
fn the_json_is_the_doctor_shape_with_command_test() {
    let document = json(&mixed().report(&Helpers::default(), &[]));
    assert_eq!(document["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(document["command"], "test");
    let part = &document["checks"][0];
    assert_eq!(part["name"], "credentials");
    assert_eq!(part["result"], "issue");
    let rows: Vec<(&str, &str)> = part["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["name"].as_str().unwrap(),
                row["result"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [("fake", "ok"), ("old", "warning"), ("work", "issue")]
    );
    assert_eq!(part["rows"][1]["advice"], "update dam-remote-old");
    assert_eq!(document["needs_attention"].as_array().unwrap().len(), 2);
    assert_matches_the_published_schema(&document);
}

pub(super) fn assert_matches_the_published_schema(document: &serde_json::Value) {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/schemas/");
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(format!("{folder}{name}")).unwrap()).unwrap()
    };
    let mut options = jsonschema::options();
    for entry in std::fs::read_dir(folder).unwrap() {
        let published = read(entry.unwrap().file_name().to_str().unwrap());
        let id = published["$id"].as_str().unwrap().to_string();
        options =
            options.with_resource(id, jsonschema::Resource::from_contents(published).unwrap());
    }
    let validator = options.build(&read("test.schema.json")).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(document)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}
