use serde_json::{Value, json};
use yaml_rust2::{Yaml, YamlLoader};

pub fn yaml_document(text: &str) -> Value {
    let documents = YamlLoader::load_from_str(text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    assert_eq!(documents.len(), 1, "not one YAML document:\n{text}");
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
