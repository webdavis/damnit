use std::path::{Path, PathBuf};

use mlua::{Lua, Value};
use serde_json::{Map, Value as Json};

use super::support::config;
use crate::lua_config::LuaConfigFile;
use crate::lua_config::evaluate::{ErrorText, evaluate};
use crate::lua_config::shape::{list, text};

fn docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs")
}

fn as_json(value: Value) -> Json {
    match value {
        Value::Boolean(b) => Json::Bool(b),
        Value::Integer(n) => Json::from(n),
        Value::Number(n) => Json::from(n),
        Value::String(_) => Json::String(text(&value).unwrap()),
        Value::Table(ref table) => match list(&value).filter(|items| !items.is_empty()) {
            Some(items) => Json::Array(items.into_iter().map(as_json).collect()),
            None => Json::Object(
                table
                    .pairs::<String, Value>()
                    .map(|pair| pair.map(|(k, v)| (k, as_json(v))).unwrap())
                    .collect::<Map<_, _>>(),
            ),
        },
        _ => Json::Null,
    }
}

fn schema(name: &str) -> Json {
    serde_json::from_str(&std::fs::read_to_string(docs().join("schemas").join(name)).unwrap())
        .unwrap()
}

fn schema_errors(example: &str, schema_name: &str) -> Vec<String> {
    let lua = Lua::new();
    let table = evaluate(
        &lua,
        &docs().join("examples").join(example),
        ErrorText::QuotesTheFile,
    )
    .unwrap()
    .unwrap();
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema(schema_name))
        .unwrap();
    validator
        .iter_errors(&as_json(Value::Table(table)))
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn the_example_files_load() {
    let config = LuaConfigFile::new(docs().join("examples/config.lua"))
        .load()
        .unwrap()
        .config;
    assert_eq!(config.owner.as_deref(), Some("Ada"));
    assert_eq!(config.remote("todoist").unwrap().credentials.len(), 1);
    assert_eq!(config.tokens.names, ["tool"]);
}

#[test]
fn the_example_files_match_their_published_schemas() {
    for (example, schema) in [
        ("config.lua", "config.schema.json"),
        ("secrets.lua", "secrets.schema.json"),
    ] {
        let errors = schema_errors(example, schema);
        assert!(errors.is_empty(), "{example} breaks {schema}: {errors:?}");
    }
}

#[test]
fn a_function_credential_is_left_open_by_the_secrets_schema() {
    let lua = Lua::new();
    let secrets: Value = lua
        .load("return { remote = { todoist = { api_token = function() return 'x' end } } }")
        .eval()
        .unwrap();
    assert!(jsonschema::is_valid(
        &schema("secrets.schema.json"),
        &as_json(secrets)
    ));
}

#[test]
fn the_reader_and_the_config_schema_refuse_the_same_remotes() {
    for (remote, accepted) in [
        ("{ url = 'todoist::', stale = '15m' }", true),
        ("{ url = 'todoist::', stale = '+15m' }", false),
        ("{ url = 'a:b::x' }", false),
        ("{ url = '../elsewhere::' }", false),
        ("{ url = 'two words::' }", false),
    ] {
        let text = format!("return {{ remote = {{ todoist = {remote} }} }}");
        let lua = Lua::new();
        let table: Value = lua.load(&text).eval().unwrap();
        let by_schema = jsonschema::is_valid(&schema("config.schema.json"), &as_json(table));
        let by_reader = config(&text).load().is_ok();
        assert_eq!((by_reader, by_schema), (accepted, accepted), "{remote}");
    }
}
