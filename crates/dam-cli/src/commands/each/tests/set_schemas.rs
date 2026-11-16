use super::as_one::a_waiting_set;
use super::*;

macro_rules! schema {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/",
            $name
        ))
    };
}

const REFERENCED: [&str; 3] = [
    schema!("status.schema.json"),
    schema!("object.schema.json"),
    schema!("settled.schema.json"),
];

fn errors(schema: &str, data: &serde_json::Value) -> Vec<String> {
    let mut options = jsonschema::options();
    for text in REFERENCED {
        let referenced: serde_json::Value = serde_json::from_str(text).unwrap();
        let id = referenced["$id"].as_str().unwrap().to_string();
        let resource = jsonschema::Resource::from_contents(referenced).unwrap();
        options = options.with_resource(id, resource);
    }
    let validator = options
        .build(&serde_json::from_str(schema).unwrap())
        .unwrap();
    validator.iter_errors(data).map(|e| e.to_string()).collect()
}

#[test]
fn status_show_and_rm_of_a_set_match_their_published_schemas() {
    let (ctx, set, _) = a_waiting_set();
    let answers = [
        (
            schema!("status.schema.json"),
            dam(&ctx, &["status"]).unwrap(),
        ),
        (
            schema!("show-set.schema.json"),
            dam(&ctx, &["show", &set]).unwrap(),
        ),
        (
            schema!("settled-set.schema.json"),
            dam(&ctx, &["rm", &set]).unwrap(),
        ),
    ];
    for (schema, answer) in answers {
        let errors = errors(schema, &answer.data);
        assert!(errors.is_empty(), "{errors:?}\n{}", answer.data);
    }
}
