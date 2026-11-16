use dam_application::{SelectError, UseCaseError};
use dam_domain::{DefaultsError, Filter, FilterTable, TableValue};

use super::Fault;
use crate::error::CliError;

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/error.schema.json"
));

fn typed(key: &str, value: &str) -> FilterTable {
    FilterTable {
        fields: [(key.to_string(), TableValue::One(value.into()))].into(),
        any: Vec::new(),
    }
}

#[test]
fn every_selection_fault_document_matches_the_published_error_schema() {
    let schema = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let bad = Filter::read(&typed("due", "tomorow")).unwrap_err();
    let conflict = DefaultsError::KindContradicts { field: "start" };
    let errors = [
        CliError::Selection(Box::new(Fault {
            error: SelectError::Value(bad.clone()),
            typed: typed("due", "tomorow"),
            sort: Vec::new(),
        })),
        CliError::Selection(Box::new(Fault {
            error: SelectError::Defaults(conflict),
            typed: typed("kind", "task"),
            sort: Vec::new(),
        })),
        CliError::UseCase(UseCaseError::Select(SelectError::Value(bad))),
    ];
    for error in errors {
        let document = error.document();
        let faults: Vec<String> = validator
            .iter_errors(&document)
            .map(|e| e.to_string())
            .collect();
        assert!(faults.is_empty(), "{document}: {faults:?}");
    }
}
