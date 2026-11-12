pub fn assert_matches_schema(document: &serde_json::Value, schema_file: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/schemas")
        .join(schema_file);
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(document)
        .map(|e| e.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "{document} breaks {schema_file}: {errors:?}"
    );
}
