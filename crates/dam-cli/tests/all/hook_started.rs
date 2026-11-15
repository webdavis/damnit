use crate::support;

use std::time::{Duration, Instant};

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn started_by_a_hook(sb: &Sandbox, args: &[&str]) -> std::process::Output {
    let missing = sb.dir.path().join("missing");
    sb.command(args)
        .env("DAM_HOOK", "task_new")
        .env("DAM_CONFIG", missing.join("config.lua"))
        .env("DAM_STORE", missing.join("dam.db"))
        .env("DAM_TOKEN", "not a token")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_dam_a_hook_started_refuses_before_it_reads_or_writes_anything() {
    let _guard = support::guard("a_dam_a_hook_started_refuses_before_it_reads_or_writes_anything");
    let sb = Sandbox::new();
    for args in [
        &["ls", "--json"][..],
        &["new", "buy milk", "--json"],
        &["status", "--json"],
    ] {
        let out = started_by_a_hook(&sb, args);
        assert_eq!(out.status.code(), Some(4), "{args:?} {out:?}");
        assert!(out.stdout.is_empty(), "{args:?} {out:?}");
        let document = error_document(&out);
        assert_matches_schema(&document, "error.schema.json");
        assert_eq!(document["error"]["rule"], "started_by_hook", "{args:?}");
        assert!(!sb.dir.path().join("missing").exists(), "{args:?}");
    }
}

#[test]
fn the_refusal_names_the_hook_that_started_dam() {
    let _guard = support::guard("the_refusal_names_the_hook_that_started_dam");
    let sb = Sandbox::new();
    let out = started_by_a_hook(&sb, &["ls"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: hook task_new started this dam, and a hook cannot run dam\n"
    );
}

#[test]
fn dam_new_inside_a_hook_refuses_at_once_and_the_hook_finishes_in_time() {
    let _guard =
        support::guard("dam_new_inside_a_hook_refuses_at_once_and_the_hook_finishes_in_time");
    let sb = Sandbox::new();
    let code = sb.dir.path().join("inner-code");
    let inner = format!(
        "{:?} new inner > /dev/null 2>&1; echo $? > {:?}",
        env!("CARGO_BIN_EXE_dam"),
        code
    );
    sb.write_config_lua(&format!(
        "return {{ hook = {{ task_new = function() os.execute({inner:?}) end }} }}\n"
    ));
    sb.write_secrets_lua("return { schema = 1 }\n");
    let started = Instant::now();
    let (ok, _, err) = sb.dam(&["new", "buy milk"]);
    assert!(ok, "{err}");
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(err, "");
    assert_eq!(std::fs::read_to_string(code).unwrap(), "4\n");
}
