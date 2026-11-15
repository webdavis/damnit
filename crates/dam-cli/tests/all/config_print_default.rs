use crate::support;

use std::path::{Path, PathBuf};
use std::process::Output;

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const OWN: &str = "return { filter = { mine = { query = 'p1' } } }\n";

fn fresh_config(sb: &Sandbox) -> PathBuf {
    sb.dir.path().join("fresh").join("dam").join("config.lua")
}

fn run(sb: &Sandbox, config: &Path, args: &[&str]) -> Output {
    sb.command(args).env("DAM_CONFIG", config).output().unwrap()
}

#[test]
fn the_printed_text_is_what_a_first_run_writes() {
    let _guard = support::guard("the_printed_text_is_what_a_first_run_writes");
    let first = Sandbox::new();
    let created = fresh_config(&first);
    assert!(run(&first, &created, &["ls"]).status.success());
    let sb = Sandbox::new();
    let out = run(&sb, &fresh_config(&sb), &["config", "print-default"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(out.stdout, std::fs::read(&created).unwrap());
}

#[test]
fn an_existing_config_is_left_as_it_was() {
    let _guard = support::guard("an_existing_config_is_left_as_it_was");
    let sb = Sandbox::new();
    sb.write_config_lua(OWN);
    let secrets = std::fs::read(sb.secrets_path()).unwrap();
    let modified = std::fs::metadata(sb.config_path())
        .unwrap()
        .modified()
        .unwrap();
    let out = sb.output(&["config", "print-default"]);
    assert!(out.status.success(), "{out:?}");
    assert_ne!(out.stdout, OWN.as_bytes());
    assert_eq!(std::fs::read_to_string(sb.config_path()).unwrap(), OWN);
    assert_eq!(std::fs::read(sb.secrets_path()).unwrap(), secrets);
    let after = std::fs::metadata(sb.config_path())
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(after, modified);
}

#[test]
fn no_file_is_created_when_there_is_no_config() {
    let _guard = support::guard("no_file_is_created_when_there_is_no_config");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    for format in [&[][..], &["--json"], &["--toon"]] {
        let out = run(
            &sb,
            &config,
            &[&["config", "print-default"][..], format].concat(),
        );
        assert!(out.status.success(), "{format:?}: {out:?}");
        assert!(!config.parent().unwrap().exists(), "{format:?}");
    }
}

#[test]
fn a_machine_reader_gets_the_text_in_one_field() {
    let _guard = support::guard("a_machine_reader_gets_the_text_in_one_field");
    let sb = Sandbox::new();
    let plain = sb.output(&["config", "print-default"]);
    let out = sb.output(&["config", "print-default", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-print-default.schema.json");
    assert_eq!(answer["text"].as_str().unwrap().as_bytes(), plain.stdout);

    let (ok, out, err) = sb.dam(&["config", "print-default", "--toon"]);
    assert!(ok, "{err}");
    assert!(out.starts_with("text: "), "{out}");
}

#[test]
fn a_token_is_refused_before_anything_is_read_or_written() {
    let _guard = support::guard("a_token_is_refused_before_anything_is_read_or_written");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    let out = sb
        .command(&["config", "print-default", "--json"])
        .env("DAM_CONFIG", &config)
        .env("DAM_TOKEN", "damk_v1_whatever")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
    assert!(!config.parent().unwrap().exists());
}

#[test]
fn config_takes_only_its_own_commands() {
    let _guard = support::guard("config_takes_only_its_own_commands");
    let sb = Sandbox::new();
    for line in [
        &["config", "check"][..],
        &["config", "print-default", "extra"],
    ] {
        let out = sb.output(line);
        assert_eq!(out.status.code(), Some(2), "{line:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{line:?}: {out:?}");
    }
}
