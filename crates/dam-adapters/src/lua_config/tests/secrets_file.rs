use dam_application::{CredentialSpec, RemoteName};

use super::support::{config, secrets};

fn credentials(text: &str) -> Vec<CredentialSpec> {
    let config = secrets(text).load().unwrap();
    config.remote("todoist").unwrap().credentials.clone()
}

#[test]
fn a_string_credential_is_attached_to_its_remote_as_a_literal() {
    let specs = credentials("return { remote = { todoist = { api_token = 'abc' } } }");
    match &specs[..] {
        [CredentialSpec::Literal { name, value }] => {
            assert_eq!(name, "api_token");
            assert_eq!(value.expose(), "abc");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_function_credential_is_attached_as_one_the_config_computes() {
    let specs = credentials(
        "return { remote = { todoist = { api_token = function() return 'abc' end } } }",
    );
    assert_eq!(
        specs,
        [CredentialSpec::Computed {
            remote: RemoteName("todoist".into()),
            name: "api_token".into()
        }]
    );
}

#[test]
fn loading_never_calls_a_function_credential() {
    let folder = secrets("");
    let marker = folder.marker();
    let folder = folder.with_secrets(&super::support::marking_credential(&marker));
    folder.load().unwrap();
    assert!(!marker.exists(), "the function ran during load");
}

#[test]
fn a_credential_that_is_neither_a_string_nor_a_function_is_refused_by_name() {
    let why = secrets("return { remote = { todoist = { api_token = 42 } } }").refusal();
    assert_eq!(
        why,
        "config: api_token must be a string or a function in remote.todoist of secrets.lua"
    );
}

#[test]
fn a_remote_that_config_lua_does_not_declare_is_refused_by_name() {
    let why = secrets("return { remote = { gcal = { api_token = 'abc' } } }").refusal();
    assert!(why.contains("gcal"), "{why}");
    assert!(why.contains("config.lua"), "{why}");
}

#[test]
fn an_unknown_top_level_key_in_secrets_lua_is_refused() {
    assert_eq!(
        secrets("return { owner = 'Ada' }").refusal(),
        "config: unknown key owner at the top level of secrets.lua"
    );
}

#[test]
fn a_syntax_error_in_secrets_lua_names_that_file_and_line() {
    let why = secrets("return {\n\n  schema = ,\n}\n").refusal();
    assert!(why.contains("secrets.lua:3:"), "{why}");
}

#[test]
fn without_secrets_lua_there_are_no_tokens_and_no_credentials() {
    let config = config("return { remote = { todoist = { url = 'todoist::' } } }")
        .load()
        .unwrap();
    assert!(config.tokens.entries.is_empty());
    assert!(config.remote("todoist").unwrap().credentials.is_empty());
}

#[test]
fn a_syntax_error_in_secrets_lua_never_quotes_the_text_it_stopped_at() {
    let why = secrets("return {\n  remote = { todoist = { api_token = \"hunter2secret } },\n}\n")
        .refusal();
    assert!(why.contains("secrets.lua:2:"), "{why}");
    assert!(!why.contains("hunter2secret"), "{why}");
}

#[test]
fn a_runtime_error_in_secrets_lua_names_its_path_and_line_and_quotes_nothing_from_it() {
    for failing in ["({}).hunter2secret.value", "error('hunter2secret')"] {
        let folder = secrets(&format!(
            "return {{\n  remote = {{\n    todoist = {{ api_token = {failing} }},\n  }},\n}}\n"
        ));
        let why = folder.refusal();
        let at = format!("{}:3:", folder.path("secrets.lua").display());
        assert!(why.contains(&at), "{why}");
        assert!(!why.contains("hunter2secret"), "{why}");
    }
}
