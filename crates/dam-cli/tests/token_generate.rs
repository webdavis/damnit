mod support;

use std::path::{Path, PathBuf};

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn generated(sb: &Sandbox) -> String {
    let (ok, out, err) = sb.dam(&["token", "generate"]);
    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    out.trim_end().to_string()
}

fn empty_dir(sb: &Sandbox) -> PathBuf {
    let dir = sb.dir.path().join("empty");
    std::fs::create_dir(&dir).unwrap();
    dir
}

fn in_dir(sb: &Sandbox, dir: &Path, args: &[&str]) -> std::process::Command {
    let mut command = sb.command(args);
    command
        .env("DAM_CONFIG", dir.join("config.toml"))
        .env("DAM_STORE", dir.join("dam.db"));
    command
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect()
}

fn granting_config_read(sb: &Sandbox, value: &str) {
    sb.add_config(&format!(
        "[token.tool]\nvalue = \"{value}\"\n[[token.tool.grant]]\nallow = [\"config:read\"]\n"
    ));
}

#[test]
fn text_prints_one_token_whose_checksum_holds() {
    let _guard = support::guard("text_prints_one_token_whose_checksum_holds");
    let sb = Sandbox::new();
    let value = generated(&sb);
    assert!(value.starts_with("damk_v1_"), "{value}");
    assert!(Token::parse(&value).is_ok(), "{value}");
}

#[test]
fn two_runs_give_different_tokens() {
    let _guard = support::guard("two_runs_give_different_tokens");
    let sb = Sandbox::new();
    assert_ne!(generated(&sb), generated(&sb));
}

#[test]
fn generating_writes_nothing_where_no_config_exists() {
    let _guard = support::guard("generating_writes_nothing_where_no_config_exists");
    let sb = Sandbox::new();
    let dir = empty_dir(&sb);
    for flag in [None, Some("--json"), Some("--toon")] {
        let mut args = vec!["token", "generate"];
        args.extend(flag);
        let out = in_dir(&sb, &dir, &args).output().unwrap();
        assert!(out.status.success(), "{out:?}");
        assert!(out.stderr.is_empty(), "{out:?}");
    }
    assert_eq!(entries(&dir), Vec::<PathBuf>::new());
}

#[test]
fn a_caller_holding_a_token_is_refused_and_nothing_is_written() {
    let _guard = support::guard("a_caller_holding_a_token_is_refused_and_nothing_is_written");
    let sb = Sandbox::new();
    let held = generated(&sb);
    granting_config_read(&sb, &held);
    let dir = empty_dir(&sb);
    for presented in [held.as_str(), "not-a-token", ""] {
        let out = sb.output_holding(presented, &["token", "generate", "--json"]);
        assert_eq!(out.status.code(), Some(4), "{out:?}");
        assert!(out.stdout.is_empty());
        let document = error_document(&out);
        assert_matches_schema(&document, "error.schema.json");
        assert_eq!(document["error"]["rule"], "not_open_to_tokens");
        let out = in_dir(&sb, &dir, &["token", "generate"])
            .env("DAM_TOKEN", presented)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "{out:?}");
    }
    assert_eq!(entries(&dir), Vec::<PathBuf>::new());
}

#[test]
fn a_generated_value_in_a_config_table_authenticates_with_its_grant() {
    let _guard = support::guard("a_generated_value_in_a_config_table_authenticates_with_its_grant");
    let sb = Sandbox::new();
    let value = generated(&sb);
    granting_config_read(&sb, &value);
    let out = sb.output_holding(&value, &["category", "list", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let out = sb.output_holding(&value, &["log", "--json"]);
    assert_eq!(error_document(&out)["error"]["rule"], "not_permitted");
}

#[test]
fn json_and_toon_give_the_value_and_the_key_id_token_list_shows() {
    let _guard = support::guard("json_and_toon_give_the_value_and_the_key_id_token_list_shows");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["token", "generate", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&answer, "token-generate.schema.json");
    let value = answer["value"].as_str().unwrap();
    assert_eq!(
        answer["key_id"],
        Token::parse(value).unwrap().key_id().as_str()
    );
    granting_config_read(&sb, value);
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    let row = listed["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "tool")
        .unwrap();
    assert_eq!(row["key_id"], answer["key_id"]);
    assert_eq!(row["standing"], "active");
    let (ok, out, err) = sb.dam(&["token", "generate", "--toon"]);
    assert!(ok, "{err}");
    for fact in ["value: damk_v1_", "key_id: "] {
        assert!(out.contains(fact), "{fact} missing from {out}");
    }
}
