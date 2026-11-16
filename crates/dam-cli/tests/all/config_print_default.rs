use crate::support;

use std::path::{Path, PathBuf};
use std::process::Output;

use support::errors::error_document;
use support::sandbox::Sandbox;

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
    assert!(run(&first, &created, &["list"]).status.success());
    let sb = Sandbox::new();
    let out = run(&sb, &fresh_config(&sb), &["config", "print-default"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(out.stdout, std::fs::read(&created).unwrap());
}

#[test]
fn a_token_is_refused_before_anything_is_read_or_written() {
    let _guard = support::guard("a_token_is_refused_before_anything_is_read_or_written");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    let out = sb
        .command(&["config", "print-default", "-o", "json"])
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
