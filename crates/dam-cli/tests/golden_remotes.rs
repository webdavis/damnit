mod support;

use std::os::unix::fs::PermissionsExt;

use support::fixture::{self, OAT_MILK, run, short};
use support::golden::{assert_golden, assert_golden_output, normalize};

#[test]
fn push_prints_the_golden_document() {
    assert_golden_output(&fixture::committed(), &["--json", "push"], "push.json");
}

#[test]
fn pull_prints_the_golden_document() {
    assert_golden_output(&fixture::committed(), &["--json", "pull"], "pull.json");
}

#[test]
fn resolve_prints_the_golden_document() {
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    let args = ["--json", "resolve", &short(OAT_MILK), "--theirs"];
    assert_golden_output(&sb, &args, "resolve.json");
}

#[test]
fn remote_add_prints_the_golden_document() {
    let sb = fixture::seeded();
    let config = sb.dir.path().join("config.toml");
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o644)).unwrap();
    let args = ["--json", "remote", "add", "calendar", "gcal::primary"];
    assert_golden_output(&sb, &args, "remote_add.json");
}

#[test]
fn a_refusal_prints_the_golden_error_document_on_stderr() {
    let sb = fixture::committed();
    let out = sb.output(&["--json", "show", "9999"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_golden("error.json", &normalize(&stderr, &fixture::seeded_oids()));
}
