mod support;

use serde_json::Value;
use support::config_files::Remote;
use support::fixture::{self, OAT_MILK, run, short};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn syncing_every_minute() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("1m"));
    sb
}

fn sync_failures(sb: &Sandbox) -> Vec<Value> {
    let status = support::status_json(sb, &["--json"]);
    assert_matches_schema(&status, "status.schema.json");
    let notices = status["notices"].as_array().unwrap().iter();
    notices
        .filter(|n| n["kind"] == "sync_failed")
        .cloned()
        .collect()
}

#[test]
fn sync_alone_prints_the_help_of_its_commands() {
    let _guard = support::guard("sync_alone_prints_the_help_of_its_commands");
    let sb = Sandbox::new();
    let alone = sb.output(&["sync"]);
    assert_eq!(alone.status.code(), Some(0), "{alone:?}");
    let asked = sb.output(&["sync", "--help"]);
    assert_eq!(alone.stdout, asked.stdout);
    assert!(String::from_utf8_lossy(&asked.stdout).contains("run"));
}

#[test]
fn a_due_remote_is_pulled_and_pushed_then_not_due_until_its_interval_passes() {
    let _guard =
        support::guard("a_due_remote_is_pulled_and_pushed_then_not_due_until_its_interval_passes");
    let sb = syncing_every_minute();
    sb.new_object(&["call Bob"]);
    sb.dam(&["add", "-A"]);
    sb.dam(&["commit", "-m", "m"]);

    let (ok, out, err) = sb.dam(&["sync", "run"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "✓ fake: synced\n  pulled 1 new, 0 updated, 0 unchanged, 0 conflicts, 0 removed upstream\n  \
         pushed 1 sent, 1 ok, 0 failed, 0 skipped\n"
    );
    assert_eq!(sb.pushes().len(), 1);

    let out = sb.output(&["sync", "run", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "sync-run.schema.json");
    assert_eq!(
        answer["remotes"][0],
        serde_json::json!({"remote": "fake", "sync_every": "1m", "result": "not_due"})
    );
    assert_eq!(sb.pushes().len(), 1, "a remote not due is left alone");
}

#[test]
fn a_run_with_no_remote_syncing_says_so_and_exits_0() {
    let _guard = support::guard("a_run_with_no_remote_syncing_says_so_and_exits_0");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["sync", "run"]);
    assert!(ok, "{err}");
    assert_eq!(out, "· no remote has sync_every\n");
}

#[test]
fn a_failed_sync_exits_1_and_leaves_one_notice_until_a_sync_works() {
    let _guard = support::guard("a_failed_sync_exits_1_and_leaves_one_notice_until_a_sync_works");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::at("missing::").sync_every("1m"));

    for _ in 0..2 {
        let out = sb.output(&["sync", "run", "--json"]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_matches_schema(&answer, "sync-run.schema.json");
        assert_eq!(answer["remotes"][0]["result"], "failed", "{answer}");
    }
    let failures = sync_failures(&sb);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0]["remote"], "fake");
    let (_, status, _) = sb.dam(&["status"]);
    assert!(status.contains("sync with fake failed: "), "{status}");

    sb.add_remote("fake", Remote::fake().sync_every("1m"));
    let (ok, _, err) = sb.dam(&["sync", "run"]);
    assert!(ok, "{err}");
    assert_eq!(sync_failures(&sb), Vec::<Value>::new());
}

#[test]
fn a_sync_says_how_many_objects_it_held_for_a_conflict() {
    let _guard = support::guard("a_sync_says_how_many_objects_it_held_for_a_conflict");
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: Value = serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    sb.add_remote("fake", Remote::fake().sync_every("1m"));

    let (ok, out, err) = sb.dam(&["sync", "run"]);

    assert!(ok, "{err}");
    assert!(
        out.ends_with("\n  1 held for a conflict; see dam status, settle with dam resolve\n"),
        "{out}"
    );
}

#[test]
fn a_token_is_refused_dam_sync_run() {
    let _guard = support::guard("a_token_is_refused_dam_sync_run");
    let sb = syncing_every_minute();
    let out = sb.output_holding("damk_v1_whatever", &["sync", "run", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    let document = support::errors::error_document(&out);
    assert_eq!(document["error"]["rule"], "not_open_to_tokens");
}
