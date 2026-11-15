mod support;

use support::errors::error_document;
use support::sandbox::Sandbox;

#[test]
fn a_named_check_runs_alone() {
    let _guard = support::guard("a_named_check_runs_alone");
    let sb = Sandbox::new();
    let (code, document) = sb.doctor_json(&["doctor", "config"]);
    assert_eq!(code, Some(0));
    let names: Vec<&str> = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|check| check["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["config"]);
}

#[test]
fn an_unknown_check_is_a_usage_error_naming_the_valid_ones() {
    let _guard = support::guard("an_unknown_check_is_a_usage_error_naming_the_valid_ones");
    let sb = Sandbox::new();
    let out = sb.output(&["doctor", "bogus"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("bogus") && said.contains("config"), "{said}");

    let out = sb.output(&["doctor", "bogus", "--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(error_document(&out)["error"]["kind"], "usage");
}

#[test]
fn the_help_lists_every_check_by_name() {
    let _guard = support::guard("the_help_lists_every_check_by_name");
    let sb = Sandbox::new();
    let (ok, out, _) = sb.dam(&["doctor", "--help"]);
    assert!(ok);
    let out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(out.contains("contact nothing"), "{out}");
    assert!(
        out.contains("[possible values: config, database, helpers, credentials]"),
        "{out}"
    );
    assert!(out.contains("dam doctor config"), "{out}");
}
