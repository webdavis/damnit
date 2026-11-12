mod support;

use std::process::Stdio;

use serde_json::Value;
use support::page_examples::{Example, examples};
use support::sandbox::Sandbox;

fn page() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/agents.md");
    std::fs::read_to_string(path).unwrap()
}

fn token_value(sb: &Sandbox, name: &str) -> String {
    let config = std::fs::read_to_string(sb.dir.path().join("config.toml")).unwrap();
    let table: toml::Table = toml::from_str(&config).unwrap();
    table["token"][name]["value"].as_str().unwrap().to_string()
}

fn filled(example: &Example, answer: Option<&Value>) -> String {
    let mut script = example.script.clone();
    for key in ["hash", "oid"] {
        let placeholder = format!("<{key}>");
        if script.contains(&placeholder) {
            let answer = answer.unwrap_or_else(|| {
                panic!("line {}: {placeholder} before any answer", example.line)
            });
            script = script.replace(&placeholder, answer[key].as_str().unwrap());
        }
    }
    script
}

fn last_answer(stdout: &[u8]) -> Option<Value> {
    serde_json::Deserializer::from_slice(stdout)
        .into_iter::<Value>()
        .map_while(Result::ok)
        .filter(|document| document.get("answer").is_some())
        .last()
}

#[test]
fn every_example_on_the_agents_page_exits_as_the_page_says() {
    let _guard = support::guard("every_example_on_the_agents_page_exits_as_the_page_says")
        .allow_slow("it runs every example the page shows, one dam process each");
    let sb = Sandbox::new();
    sb.add_config("[remote.calendar]\nurl = \"fake::\"\n");
    let examples = examples(&page());
    assert!(examples.len() >= 15, "found {} examples", examples.len());
    let mut answer = None;
    for example in &examples {
        assert!(example.script.contains("dam "), "line {}", example.line);
        let script = filled(example, answer.as_ref());
        let mut command = sb.shell(&script);
        if let Some(name) = &example.holder {
            command.env("DAM_TOKEN", token_value(&sb, name));
        }
        let out = command.stdin(Stdio::null()).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(example.exit),
            "line {}: {script}\n{out:?}",
            example.line
        );
        if let Some(rule) = &example.rule {
            let error: Value = serde_json::from_slice(&out.stderr).unwrap();
            assert_eq!(
                error["error"]["rule"],
                rule.as_str(),
                "line {}",
                example.line
            );
        }
        answer = last_answer(&out.stdout).or(answer);
    }
}
