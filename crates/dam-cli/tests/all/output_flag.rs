use crate::support;

use support::errors::error_document;
use support::fixture::{self, OAT_MILK, short};
use support::sandbox::Sandbox;
use support::yaml::yaml_document;

const EXIT_USAGE: i32 = 2;

fn answer(sb: &Sandbox, args: &[&str]) -> String {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
    out
}

#[test]
fn every_read_answers_the_same_value_in_yaml_as_in_json() {
    let _guard = support::guard("every_read_answers_the_same_value_in_yaml_as_in_json");
    let sb = fixture::committed();
    let milk = short(OAT_MILK);
    let reads: [&[&str]; 9] = [
        &["list"],
        &["show", &milk],
        &["status", "--full"],
        &["log"],
        &["agenda"],
        &["filter", "list"],
        &["category", "list"],
        &["config", "list"],
        &["doctor"],
    ];
    for read in reads {
        let json: serde_json::Value =
            serde_json::from_str(&answer(&sb, &[read, &["-o", "json"]].concat())).unwrap();
        let yaml = yaml_document(&answer(&sb, &[read, &["-o", "yaml"]].concat()));
        assert_eq!(yaml, json, "dam {read:?}");
    }
}

#[test]
fn quiet_prints_nothing_for_a_person_and_the_answer_in_each_format() {
    let _guard = support::guard("quiet_prints_nothing_for_a_person_and_the_answer_in_each_format");
    let sb = fixture::committed();
    assert_eq!(answer(&sb, &["list", "-q"]), "");
    let json = answer(&sb, &["list", "-q", "-o", "json"]);
    let toon = answer(&sb, &["list", "-q", "-o", "toon"]);
    let yaml = answer(&sb, &["list", "-q", "-o", "yaml"]);
    assert_eq!(json, answer(&sb, &["list", "-o", "json"]));
    assert_eq!(toon, answer(&sb, &["list", "-o", "toon"]));
    assert_eq!(yaml, answer(&sb, &["list", "-o", "yaml"]));
    assert!(!toon.is_empty() && !yaml.is_empty());
}

#[test]
fn a_failure_under_yaml_is_one_yaml_error_document_on_stderr() {
    let _guard = support::guard("a_failure_under_yaml_is_one_yaml_error_document_on_stderr");
    let sb = Sandbox::new();
    for line in [&["show", "fffffff"][..], &["agenda", "--bogus"][..]] {
        let yaml = sb.output(&[line, &["-o", "yaml"]].concat());
        let json = sb.output(&[line, &["-o", "json"]].concat());
        assert_eq!(yaml.status.code(), json.status.code(), "{line:?}");
        assert!(yaml.stdout.is_empty(), "{line:?}");
        let document = yaml_document(&String::from_utf8_lossy(&yaml.stderr));
        assert_eq!(document, error_document(&json), "{line:?}");
    }
}

#[test]
fn an_unknown_output_is_a_usage_error_naming_the_three_values() {
    let _guard = support::guard("an_unknown_output_is_a_usage_error_naming_the_three_values");
    let out = Sandbox::new().output(&["list", "-o", "xml"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("expected one of: json, toon, yaml"), "{err}");
}

#[test]
fn the_removed_format_flags_are_unknown_flags_that_name_no_replacement() {
    let _guard =
        support::guard("the_removed_format_flags_are_unknown_flags_that_name_no_replacement");
    let sb = Sandbox::new();
    for removed in ["--json", "--toon"] {
        let out = sb.output(&["list", removed]);
        assert_eq!(out.status.code(), Some(EXIT_USAGE), "{removed}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(&format!("unexpected argument '{removed}'")),
            "{err}"
        );
        assert!(!err.contains("output") && !err.contains("-o "), "{err}");
    }
}
