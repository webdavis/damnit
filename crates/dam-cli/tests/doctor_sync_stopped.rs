mod support;

use std::time::{Duration, SystemTime};

use support::config_files::Remote;
use support::doctor::{ALL_WELL, report};
use support::sandbox::Sandbox;

const REINSTALL: &str = "      → run dam sync disable, then dam sync enable";

fn last_run(sb: &Sandbox, said: &str) -> String {
    let path = sb.dir.path().join("state/dam/sync-last-run.txt");
    format!("last run {}, {said}", path.display())
}

fn written_minutes_ago(sb: &Sandbox, minutes: u64) {
    let path = sb.dir.path().join("state/dam/sync-last-run.txt");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(&path).unwrap();
    let at = SystemTime::now() - Duration::from_secs(minutes * 60);
    file.set_modified(at).unwrap();
}

fn installed(sb: &Sandbox) {
    assert!(sb.scheduling(&["sync", "enable"]).status.success());
}

fn doctor_sync(sb: &Sandbox) -> String {
    sb.doctor(&["doctor", "sync"]).out
}

#[test]
fn an_installed_schedule_that_never_ran_is_a_warning_naming_the_reinstall() {
    let _guard = support::guard("an_installed_schedule_that_never_ran_is_a_warning");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("1m"));
    installed(&sb);
    let said = "the schedule is installed and has never run";
    assert_eq!(
        doctor_sync(&sb),
        report(
            &[
                "  ⚠ sync: 1 remote: 1 ok",
                &format!("    ⚠ {}", last_run(&sb, said)),
                REINSTALL,
                "    ✓ fake: every 1m, not synced yet",
            ],
            &["  ⚠ 1 warning to look at", &format!("  1. sync: {said}")]
        )
    );
}

#[test]
fn an_installed_schedule_with_no_sync_every_that_stopped_is_a_warning_naming_when() {
    let _guard = support::guard("an_installed_schedule_with_no_sync_every_that_stopped");
    let sb = Sandbox::new();
    installed(&sb);
    written_minutes_ago(&sb, 12);
    let said = "the schedule is installed and has not run since 12m ago";
    assert_eq!(
        doctor_sync(&sb),
        report(
            &[
                "  ⚠ sync: no remote has sync_every",
                &format!("    ⚠ {}", last_run(&sb, said)),
                REINSTALL,
            ],
            &["  ⚠ 1 warning to look at", &format!("  1. sync: {said}")]
        )
    );
}

#[test]
fn an_installed_schedule_that_ran_within_the_bound_is_ok() {
    let _guard = support::guard("an_installed_schedule_that_ran_within_the_bound_is_ok");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("1m"));
    installed(&sb);
    written_minutes_ago(&sb, 9);
    assert_eq!(
        doctor_sync(&sb),
        report(
            &[
                "  ✓ sync: 1 remote: 1 ok",
                &format!("    ✓ {}", last_run(&sb, "written 9m ago")),
                "    ✓ fake: every 1m, not synced yet",
            ],
            ALL_WELL
        )
    );
}

#[test]
fn an_old_last_run_without_the_schedule_adds_no_warning() {
    let _guard = support::guard("an_old_last_run_without_the_schedule_adds_no_warning");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("1m"));
    written_minutes_ago(&sb, 60 * 24);
    let not_installed = "fake: sync_every is set and the schedule is not installed";
    assert_eq!(
        doctor_sync(&sb),
        report(
            &[
                "  ⚠ sync: 1 remote: 1 warning",
                &format!("    · {}", last_run(&sb, "written 1d ago")),
                &format!("    ⚠ {not_installed}"),
                "      → run dam sync enable",
            ],
            &[
                "  ⚠ 1 warning to look at",
                &format!("  1. sync for {not_installed}")
            ]
        )
    );
}

#[test]
fn the_json_carries_the_stopped_schedule_as_the_last_run_result() {
    let _guard = support::guard("the_json_carries_the_stopped_schedule");
    let sb = Sandbox::new();
    installed(&sb);
    written_minutes_ago(&sb, 12);
    let (_, document) = sb.doctor_json(&["doctor", "sync"]);
    let check = &document["checks"][0];
    let said = "the schedule is installed and has not run since 12m ago";
    assert_eq!(check["result"], "warning");
    assert_eq!(check["last_run"]["result"], "warning");
    assert_eq!(check["last_run"]["detail"], said);
    assert_eq!(
        check["last_run"]["advice"],
        "run dam sync disable, then dam sync enable"
    );
    assert_eq!(document["needs_attention"][0], format!("sync: {said}"));
}
