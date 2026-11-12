use std::collections::BTreeSet;

use super::samples::every_refusal_with_its_text;
use crate::error::CliError;

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/error.schema.json"
));

fn schema() -> serde_json::Value {
    serde_json::from_str(SCHEMA).unwrap()
}

fn listed(schema: &serde_json::Value, def: &str) -> BTreeSet<String> {
    schema["$defs"][def]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn every_refusal_document_matches_the_published_error_schema() {
    let validator = jsonschema::validator_for(&schema()).unwrap();
    for (refusal, _) in every_refusal_with_its_text() {
        let document = CliError::from(refusal).document();
        let errors: Vec<String> = validator
            .iter_errors(&document)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{document}: {errors:?}");
    }
}

#[test]
fn the_schema_lists_every_rule_once_and_token_rules_alone_exit_five() {
    let schema = schema();
    let (rules, token_rules) = (listed(&schema, "rule"), listed(&schema, "tokenRule"));
    let mut named = BTreeSet::new();
    for (refusal, _) in every_refusal_with_its_text() {
        let error = CliError::from(refusal);
        let rule = error.rule().unwrap().to_string();
        let expected_exit = if token_rules.contains(&rule) {
            5
        } else if rule == "ambiguous_prefix" {
            2
        } else {
            4
        };
        assert_eq!(error.exit_code(), expected_exit, "{rule}");
        named.insert(rule);
    }
    assert!(rules.is_disjoint(&token_rules));
    assert_eq!(named, &rules | &token_rules);
}
