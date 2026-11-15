use crate::support;

use std::path::PathBuf;
use std::time::SystemTime;

use support::doctor::listing;
use support::sandbox::Sandbox;

fn store(sb: &Sandbox) -> PathBuf {
    sb.dir.path().join("dam.db")
}

fn facts(path: &PathBuf) -> (Vec<u8>, SystemTime, u64) {
    let meta = std::fs::metadata(path).unwrap();
    (
        std::fs::read(path).unwrap(),
        meta.modified().unwrap(),
        meta.len(),
    )
}

fn created(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let (_, document) = sb.doctor_json(args);
    document["created"].clone()
}

#[test]
fn a_config_that_does_not_read_is_left_byte_for_byte() {
    let _guard = support::guard("a_config_that_does_not_read_is_left_byte_for_byte");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let config = std::fs::read(sb.config_path()).unwrap();
    let secrets = std::fs::read(sb.secrets_path()).unwrap();
    let answer = sb.doctor(&["doctor", "--fix", "config"]);
    assert_eq!(answer.code, Some(1), "{}", answer.out);
    assert!(
        answer.out.contains("\n  · created nothing\n"),
        "{}",
        answer.out
    );
    assert_eq!(std::fs::read(sb.config_path()).unwrap(), config);
    assert_eq!(std::fs::read(sb.secrets_path()).unwrap(), secrets);
}

#[test]
fn an_existing_store_is_not_touched() {
    let _guard = support::guard("an_existing_store_is_not_touched");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    let (store_before, listing_before) = (facts(&store(&sb)), listing(sb.dir.path()));
    assert_eq!(
        created(&sb, &["doctor", "--fix", "config", "database"]),
        serde_json::json!([])
    );
    assert_eq!(listing(sb.dir.path()), listing_before);
    assert!(facts(&store(&sb)) == store_before, "the store changed");
}

#[test]
fn a_store_with_only_its_side_file_is_not_replaced() {
    let _guard = support::guard("a_store_with_only_its_side_file_is_not_replaced");
    let sb = Sandbox::new();
    let wal = sb.dir.path().join("dam.db-wal");
    std::fs::write(&wal, "left over").unwrap();
    let before = listing(sb.dir.path());
    assert_eq!(
        created(&sb, &["doctor", "--fix", "database"]),
        serde_json::json!([])
    );
    assert_eq!(listing(sb.dir.path()), before);
    assert_eq!(std::fs::read(&wal).unwrap(), b"left over");
}

#[test]
fn a_config_path_not_ending_in_lua_is_not_created() {
    let _guard = support::guard("a_config_path_not_ending_in_lua_is_not_created");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("fresh/config.toml");
    let out = sb
        .command(&["doctor", "--fix", "config", "--json"])
        .env("DAM_CONFIG", &config)
        .output()
        .unwrap();
    let document: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(document["created"], serde_json::json!([]));
    assert!(!config.exists());
}

#[test]
fn config_validate_takes_no_fix() {
    let _guard = support::guard("config_validate_takes_no_fix");
    let sb = Sandbox::new();
    let out = sb.output(&["config", "validate", "--fix"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn a_token_cannot_fix() {
    let _guard = support::guard("a_token_cannot_fix");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.config_path()).unwrap();
    let before = listing(sb.dir.path());
    let out = sb.output_holding("damk_v1_whatever", &["doctor", "--fix"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(listing(sb.dir.path()), before);
}
