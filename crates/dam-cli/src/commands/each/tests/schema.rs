use super::*;

const SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/changes.schema.json"
));
const STATUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/status.schema.json"
));
const OBJECT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/object.schema.json"
));

fn validator() -> jsonschema::Validator {
    let mut options = jsonschema::options();
    for text in [STATUS, OBJECT] {
        let schema: serde_json::Value = serde_json::from_str(text).unwrap();
        let id = schema["$id"].as_str().unwrap().to_string();
        options = options.with_resource(id, jsonschema::Resource::from_contents(schema).unwrap());
    }
    options
        .build(&serde_json::from_str(SCHEMA).unwrap())
        .unwrap()
}

#[test]
fn every_answer_matches_the_published_changes_schema() {
    let validator = validator();
    let (ctx, oids) = three_tasks();
    let answers = [
        dam(
            &ctx,
            &["edit", &oids[0], "--due", "2026-11-20", "--preview"],
        )
        .unwrap(),
        dam(&ctx, &["done", "--query", "path:garden/"]).unwrap(),
        dam(&ctx, &["rm", &oids[0], &oids[1]]).unwrap(),
    ];
    for answer in answers {
        let errors: Vec<String> = validator
            .iter_errors(&answer.data)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{errors:?}\n{}", answer.data);
    }
}
