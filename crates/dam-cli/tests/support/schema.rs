use std::path::PathBuf;

fn schemas() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/schemas")
}

fn read(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

pub fn assert_matches_schema(document: &serde_json::Value, schema_file: &str) {
    let mut options = jsonschema::options();
    for entry in std::fs::read_dir(schemas()).unwrap() {
        let published = read(&entry.unwrap().path());
        let id = published["$id"].as_str().unwrap().to_string();
        options =
            options.with_resource(id, jsonschema::Resource::from_contents(published).unwrap());
    }
    let validator = options.build(&read(&schemas().join(schema_file))).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(document)
        .map(|e| e.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "{document} breaks {schema_file}: {errors:?}"
    );
}
