use crate::support;

use serde_json::Value;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::small_config::small_config;

fn json_of(sb: &Sandbox, args: &[&str]) -> Value {
    let out = sb.output(&[args, &["--json"]].concat());
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn rows(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| line.strip_prefix("  ")?.split_once("  "))
        .map(|(key, value)| (key.to_string(), value.trim_start().to_string()))
        .collect()
}

#[test]
fn every_listed_key_gets_its_own_row() {
    let _guard = support::guard("every_listed_key_gets_its_own_row");
    let sb = small_config();
    let (ok, listing, err) = sb.dam(&["config", "list"]);
    assert!(ok, "{err}");
    let settings = listing.split_once("Settings ──").unwrap().1;
    for (key, value) in rows(settings) {
        let (ok, out, err) = sb.dam(&["config", "get", &key]);
        assert!(ok, "{key}: {err}");
        assert_eq!(rows(&out), vec![(key.clone(), value)], "{key}");
    }
}

#[test]
fn a_setting_a_part_and_a_default_print_by_key() {
    let _guard = support::guard("a_setting_a_part_and_a_default_print_by_key");
    let sb = small_config();
    let got = |key: &str| sb.dam(&["config", "get", key]).1;
    assert_eq!(got("owner"), "  owner  Ada\n");
    assert_eq!(
        got("remote.todoist.deadline"),
        "  remote.todoist.deadline  60s (default)\n"
    );
    assert_eq!(
        got("remote.todoist"),
        "  remote.todoist.url                   todoist::
  remote.todoist.deadline              60s (default)
  remote.todoist.path                  work/
  remote.todoist.credential.api_token  hidden
"
    );
    assert_eq!(
        got("remote.todoist.credential"),
        "  remote.todoist.credential.api_token  hidden\n"
    );
}

#[test]
fn a_key_that_names_nothing_is_refused_with_the_key() {
    let _guard = support::guard("a_key_that_names_nothing_is_refused_with_the_key");
    let sb = small_config();
    for key in [
        "nope",
        "config",
        "remote.gone",
        "remote.todoist.nope",
        "remote.todoist.url.more",
        "remote.todoist.api_token",
        "remote.todoist.credential.gone",
        "filter.gone",
        "category.gone",
        "token.gone",
    ] {
        let out = sb.output(&["config", "get", key]);
        assert_eq!(out.status.code(), Some(4), "{key}: {out:?}");
        assert!(out.stdout.is_empty(), "{key}");
        let said = String::from_utf8(out.stderr).unwrap();
        let message = format!("no setting named {key}; run dam config list to see the keys");
        assert_eq!(said, format!("dam: {message}\n"));
        let out = sb.output(&["config", "get", key, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{key}: {out:?}");
        let document = error_document(&out);
        assert_matches_schema(&document, "error.schema.json");
        assert_eq!(document["error"]["kind"], "refused", "{key}");
        assert_eq!(document["error"]["rule"], "no_such_setting", "{key}");
        assert_eq!(document["error"]["key"], key);
        assert_eq!(document["error"]["message"], message);
    }
}

#[test]
fn an_empty_key_is_a_usage_error() {
    let _guard = support::guard("an_empty_key_is_a_usage_error");
    let sb = small_config();
    let out = sb.output(&["config", "get", "", "--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(error_document(&out)["error"]["kind"], "usage");
    assert!(!String::from_utf8_lossy(&out.stderr).contains("no setting named"));
}

#[test]
fn a_part_answers_as_it_sits_in_the_whole_document() {
    let _guard = support::guard("a_part_answers_as_it_sits_in_the_whole_document");
    let sb = small_config();
    let whole = json_of(&sb, &["config", "list"]);
    for (key, pointer) in [
        ("remote.todoist", "/settings/remote/todoist"),
        (
            "remote.todoist.credential",
            "/settings/remote/todoist/credential",
        ),
        ("owner", "/settings/owner"),
        ("done", "/settings/done"),
        ("token.tool", "/settings/token/tool"),
    ] {
        let part = json_of(&sb, &["config", "get", key]);
        assert_matches_schema(&part, "config-get.schema.json");
        assert_eq!(&part, whole.pointer(pointer).unwrap(), "{key}");
        let (ok, out, err) = sb.dam(&["config", "get", key, "--toon"]);
        assert!(ok, "{err}");
        assert_eq!(toon_format::decode_default::<Value>(&out).unwrap(), part);
    }
}

#[test]
fn a_token_is_refused_before_the_config_is_read() {
    let _guard = support::guard("a_token_is_refused_before_the_config_is_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let out = sb.output_holding("damk_v1_whatever", &["config", "get", "owner", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
}

#[test]
fn no_config_gets_a_default_and_creates_nothing() {
    let _guard = support::guard("no_config_gets_a_default_and_creates_nothing");
    let sb = Sandbox::new();
    let folder = sb.dir.path().join("fresh");
    let out = sb
        .command(&["config", "get", "owner"])
        .env("DAM_CONFIG", folder.join("config.lua"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(out.stdout, b"  owner  owner (default)\n");
    assert!(!folder.exists());
}
