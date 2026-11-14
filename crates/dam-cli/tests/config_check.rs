mod support;

use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::TokenTable;

const RAISES: &str = "return { schema = 1, token = error('the vault said hunter2') }\n";

#[test]
fn good_files_answer_with_their_paths_and_the_token_warnings() {
    let _guard = support::guard("good_files_answer_with_their_paths_and_the_token_warnings");
    let sb = Sandbox::new();
    sb.add_token("tool", TokenTable::valued("not-a-token"));
    let (ok, out, err) = sb.dam(&["config", "check"]);
    assert!(ok, "{err}");
    let config = sb.config_path().display().to_string();
    let secrets = sb.secrets_path().display().to_string();
    assert_eq!(out, format!("{config}\n{secrets}\n"));
    assert!(
        err.starts_with("warning: ") && err.contains("not a damnit token"),
        "{err}"
    );

    let out = sb.output(&["config", "check", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-check.schema.json");
    assert_eq!(answer["config"], config.as_str());
    assert_eq!(answer["secrets"], secrets.as_str());
    assert_eq!(answer["warnings"].as_array().unwrap().len(), 1, "{answer}");

    let (ok, out, err) = sb.dam(&["config", "check", "--toon"]);
    assert!(ok, "{err}");
    assert!(out.contains("warnings[1]"), "{out}");
}

#[test]
fn a_token_is_refused_before_the_config_is_read() {
    let _guard = support::guard("a_token_is_refused_before_the_config_is_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let out = sb.output_holding("damk_v1_whatever", &["config", "check", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
}

#[test]
fn a_lua_error_in_config_lua_names_its_line_and_the_check() {
    let _guard = support::guard("a_lua_error_in_config_lua_names_its_line_and_the_check");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    let config = sb.config_path().display().to_string();
    let (ok, out, err) = sb.dam(&["ls", "--no-pull"]);
    assert!(!ok && out.is_empty());
    assert!(err.contains(&format!("{config}:2:")), "{err}");
    assert!(err.contains("dam config check"), "{err}");

    let out = sb.output(&["config", "check", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(kind_of(&out), "parse");
    let message = error_document(&out)["error"]["message"].to_string();
    assert!(message.contains(&format!("{config}:2:")), "{message}");
}

#[test]
fn a_runtime_error_in_secrets_lua_is_quoted_by_the_check_alone() {
    let _guard = support::guard("a_runtime_error_in_secrets_lua_is_quoted_by_the_check_alone");
    let sb = Sandbox::new();
    sb.write_secrets_lua(RAISES);
    let secrets = sb.secrets_path().display().to_string();
    let (ok, out, err) = sb.dam(&["ls", "--no-pull"]);
    assert!(!ok && out.is_empty());
    assert!(
        err.contains(&format!("{secrets}:1: loading stopped at an error")),
        "{err}"
    );
    assert!(err.contains("dam config check"), "{err}");
    assert!(!err.contains("hunter2"), "{err}");

    let (ok, out, err) = sb.dam(&["config", "check"]);
    assert!(!ok && out.is_empty());
    assert!(err.contains(&format!("{secrets}:1:")), "{err}");
    assert!(err.contains("the vault said hunter2"), "{err}");
}

#[test]
fn a_syntax_error_in_secrets_lua_is_never_quoted() {
    let _guard = support::guard("a_syntax_error_in_secrets_lua_is_never_quoted");
    let sb = Sandbox::new();
    sb.write_secrets_lua("return { token = { tool = { value = 'hunter2' } } \n");
    for line in [&["ls", "--no-pull"][..], &["config", "check"]] {
        let (ok, out, err) = sb.dam(line);
        assert!(!ok && out.is_empty(), "{line:?}");
        assert!(err.contains("secrets.lua:"), "{line:?}: {err}");
        assert!(!err.contains("hunter2"), "{line:?}: {err}");
    }
}
