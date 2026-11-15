mod support;

use support::doctor::{ALL_WELL, listing, report};
use support::sandbox::Sandbox;

fn commit_two_tasks(sb: &Sandbox) {
    for subject in ["buy oat milk", "call Ada"] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", subject]).0);
    }
}

#[test]
fn a_machine_set_up_well_reads_every_check_and_changes_nothing() {
    let _guard = support::guard("a_machine_set_up_well_reads_every_check_and_changes_nothing");
    let sb = Sandbox::new();
    commit_two_tasks(&sb);
    assert!(sb.dam(&["push"]).0);
    let before = listing(sb.dir.path());
    let answer = sb.doctor(&["doctor"]);
    assert_eq!(answer.code, Some(0));
    assert_eq!(
        answer.out,
        report(
            &[
                "  ✓ config: both files are valid",
                "  ✓ database: version 12",
                "  ✓ helpers: 1 remote: 1 ok",
                "    ✓ fake: dam-remote-fake answered",
                "  ✓ credentials: 1 remote: 1 ok",
                "    ✓ fake: has its credential, not tested",
                "      → run dam pull fake to try it",
                "  ✓ pushes: 1 remote: 1 ok",
                "    ✓ fake: nothing owed",
            ],
            ALL_WELL
        )
    );
    assert_eq!(listing(sb.dir.path()), before);
}

#[test]
fn an_issue_and_a_warning_are_counted_then_numbered() {
    let _guard = support::guard("an_issue_and_a_warning_are_counted_then_numbered");
    let sb = Sandbox::new();
    commit_two_tasks(&sb);
    std::fs::remove_file(sb.dir.path().join("bin/dam-remote-fake")).unwrap();
    let answer = sb.doctor(&["doctor"]);
    assert_eq!(answer.code, Some(1));
    assert_eq!(
        answer.out,
        report(
            &[
                "  ✓ config: both files are valid",
                "  ✓ database: version 12",
                "  ✗ helpers: 1 remote: 1 issue",
                "    ✗ fake: dam-remote-fake is not installed",
                "      → install it, then run dam doctor",
                "  · credentials: 1 remote: 1 not checked",
                "    · fake: not checked, its helper did not answer",
                "  ⚠ pushes: 1 remote: 1 warning",
                "    ⚠ fake: 2 changes still owed",
                "      → run dam push",
            ],
            &[
                "  ✗ 1 issue to fix, 1 warning to look at",
                "  1. helpers for fake: dam-remote-fake is not installed",
                "  2. pushes for fake: 2 changes still owed",
            ]
        )
    );

    let (code, document) = sb.doctor_json(&["doctor"]);
    assert_eq!(code, Some(1));
    let results: Vec<(&str, &str)> = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["name"].as_str().unwrap(), c["result"].as_str().unwrap()))
        .collect();
    assert_eq!(
        results,
        [
            ("config", "ok"),
            ("database", "ok"),
            ("helpers", "issue"),
            ("credentials", "not_run"),
            ("pushes", "warning"),
        ]
    );
    assert_eq!(
        document["checks"][2]["rows"][0]["advice"],
        "install it, then run dam doctor"
    );
    assert_eq!(
        document["needs_attention"],
        serde_json::json!([
            "helpers for fake: dam-remote-fake is not installed",
            "pushes for fake: 2 changes still owed"
        ])
    );
}

#[test]
fn a_broken_config_leaves_only_the_database_to_check() {
    let _guard = support::guard("a_broken_config_leaves_only_the_database_to_check");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    let answer = sb.doctor(&["doctor"]);
    assert_eq!(answer.code, Some(1));
    let rows: Vec<&str> = answer.out.lines().skip(5).take(4).collect();
    assert_eq!(
        rows,
        [
            "  ✓ database: none yet",
            "  · helpers: not checked, the config did not read",
            "  · credentials: not checked, the config did not read",
            "  · pushes: not checked, the config did not read",
        ]
    );
    assert!(!sb.dir.path().join("token.txt").exists());
}
