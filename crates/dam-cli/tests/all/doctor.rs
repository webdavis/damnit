use crate::support;

use support::doctor::{ALL_WELL, VERSION, listing, report};
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::token_table::TokenTable;

const RAISES: &str = "return { schema = 1, token = error('the vault said hunter2') }\n";
const FIX: &str = "    → fix the file it names";

#[test]
fn good_files_read_as_one_good_row() {
    let _guard = support::guard("good_files_read_as_one_good_row");
    let sb = Sandbox::new();
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(0));
    assert_eq!(
        answer.out,
        report(&["  ✓ config: both files are valid"], ALL_WELL)
    );

    let (code, document) = sb.doctor_json(&["doctor", "config"]);
    assert_eq!(code, Some(0));
    assert_eq!(document["version"], VERSION);
    assert_eq!(document["command"], "doctor");
    assert_eq!(document["needs_attention"], serde_json::json!([]));
    let config = &document["checks"][0];
    assert_eq!(config["name"], "config");
    assert_eq!(config["result"], "ok");
    assert_eq!(config["detail"], "both files are valid");
    assert_eq!(config["advice"], serde_json::Value::Null);
    assert_eq!(config["config"], sb.config_path().display().to_string());
    assert_eq!(config["secrets"], sb.secrets_path().display().to_string());
}

#[test]
fn a_token_warning_shows_under_a_warning_row() {
    let _guard = support::guard("a_token_warning_shows_under_a_warning_row");
    let sb = Sandbox::new();
    sb.add_token("tool", TokenTable::valued("not-a-token"));
    let row = "config: both files read, with 1 token warning";
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(0));
    assert_eq!(
        answer.out,
        report(
            &[
                &format!("  ⚠ {row}"),
                "    ⚠ token tool is ignored: its value is not a damnit token",
            ],
            &["  ⚠ 1 warning to look at", &format!("  1. {row}")]
        )
    );

    let (code, document) = sb.doctor_json(&["doctor", "config"]);
    assert_eq!(code, Some(0));
    assert_eq!(document["checks"][0]["result"], "warning");
    assert_eq!(
        document["checks"][0]["warnings"],
        serde_json::json!(["token tool is ignored: its value is not a damnit token"])
    );
    assert_eq!(document["needs_attention"], serde_json::json!([row]));

    let answer = sb.doctor(&["doctor", "config", "--toon"]);
    assert!(answer.out.contains("warnings[1]"), "{}", answer.out);
}

#[test]
fn a_broken_config_lua_is_an_issue_reported_on_standard_output() {
    let _guard = support::guard("a_broken_config_lua_is_an_issue_reported_on_standard_output");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    let config = sb.config_path().display().to_string();
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(1));
    let lines: Vec<&str> = answer.out.lines().collect();
    assert!(
        lines[3].starts_with(&format!("  ✗ config: {config}:2:")),
        "{}",
        answer.out
    );
    assert_eq!(lines[4], FIX);
    assert!(
        answer.out.contains("\n  ✗ 1 issue to fix\n  1. config: "),
        "{}",
        answer.out
    );

    let (code, document) = sb.doctor_json(&["doctor", "config"]);
    assert_eq!(code, Some(1));
    assert_eq!(document["checks"][0]["result"], "issue");
    assert!(document.get("error").is_none());
}

#[test]
fn a_runtime_error_in_secrets_lua_is_quoted_in_the_row() {
    let _guard = support::guard("a_runtime_error_in_secrets_lua_is_quoted_in_the_row");
    let sb = Sandbox::new();
    sb.write_secrets_lua(RAISES);
    let secrets = sb.secrets_path().display().to_string();
    let (ok, out, err) = sb.dam(&["ls"]);
    assert!(!ok && out.is_empty());
    assert!(
        err.contains("dam doctor") && !err.contains("hunter2"),
        "{err}"
    );

    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(1));
    let quoted = format!("  ✗ config: {secrets}:1: the vault said hunter2");
    assert!(answer.out.contains(&quoted), "{}", answer.out);
}

#[test]
fn a_syntax_error_in_secrets_lua_is_never_quoted() {
    let _guard = support::guard("a_syntax_error_in_secrets_lua_is_never_quoted");
    let sb = Sandbox::new();
    sb.write_secrets_lua("return { token = { tool = { value = 'hunter2' } } \n");
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(1));
    assert!(answer.out.contains("secrets.lua:"), "{}", answer.out);
    assert!(!answer.out.contains("hunter2"), "{}", answer.out);
}

#[test]
fn no_config_lua_means_defaults_and_doctor_creates_nothing() {
    let _guard = support::guard("no_config_lua_means_defaults_and_doctor_creates_nothing");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.config_path()).unwrap();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    let before = listing(sb.dir.path());
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(0));
    assert!(
        answer
            .out
            .contains("\n  ✓ config: no config.lua, defaults in use\n"),
        "{}",
        answer.out
    );
    assert_eq!(listing(sb.dir.path()), before);
}

#[test]
fn a_token_is_refused_before_the_config_is_read() {
    let _guard = support::guard("a_token_is_refused_before_the_config_is_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let out = sb.output_holding("damk_v1_whatever", &["doctor", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
}

#[test]
fn config_check_is_an_unknown_command() {
    let _guard = support::guard("config_check_is_an_unknown_command");
    let sb = Sandbox::new();
    let out = sb.output(&["config", "check"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
}
