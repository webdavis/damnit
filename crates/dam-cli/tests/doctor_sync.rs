mod support;

use support::config_files::Remote;
use support::doctor::listing;
use support::sandbox::Sandbox;

fn rows(sb: &Sandbox) -> (Option<i32>, Vec<String>) {
    let answer = sb.doctor(&["doctor", "sync"]);
    let rows = answer
        .out
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    (answer.code, rows)
}

fn last_run(sb: &Sandbox, mark: &str, said: &str) -> String {
    let path = sb.dir.path().join("state/dam/sync-last-run.txt");
    format!("    {mark} last run {}, {said}", path.display())
}

fn enabled(remote: Remote) -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("fake", remote);
    assert!(sb.scheduling(&["sync", "enable"]).status.success());
    sb
}

#[test]
fn no_remote_with_sync_every_is_not_run_and_names_the_last_run_file() {
    let _guard = support::guard("no_remote_with_sync_every_is_not_run");
    let sb = Sandbox::new();
    assert_eq!(
        rows(&sb),
        (
            Some(0),
            vec![
                "  · sync: no remote has sync_every".to_string(),
                last_run(&sb, "·", "not written yet"),
            ]
        )
    );
}

#[test]
fn sync_every_without_the_schedule_is_a_warning_naming_dam_sync_enable() {
    let _guard = support::guard("sync_every_without_the_schedule_is_a_warning");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("15m"));
    assert_eq!(
        rows(&sb),
        (
            Some(0),
            vec![
                "  ⚠ sync: 1 remote: 1 warning".to_string(),
                last_run(&sb, "·", "not written yet"),
                "    ⚠ fake: sync_every is set and the schedule is not installed".to_string(),
                "      → run dam sync enable".to_string(),
            ]
        )
    );
}

#[test]
fn a_remote_never_synced_under_the_schedule_says_so() {
    let _guard = support::guard("a_remote_never_synced_under_the_schedule_says_so");
    let sb = enabled(Remote::fake().sync_every("15m"));
    assert_eq!(
        rows(&sb).1.last().map(String::as_str),
        Some("    ✓ fake: every 15m, not synced yet")
    );
}

#[test]
fn a_synced_remote_says_when_read_without_a_write() {
    let _guard = support::guard("a_synced_remote_says_when_read_without_a_write");
    let sb = enabled(Remote::fake().sync_every("1m"));
    let ran = sb.output(&["sync", "run", "--record-last-run"]);
    assert!(ran.status.success(), "{ran:?}");
    let before = listing(sb.dir.path());
    assert_eq!(
        rows(&sb),
        (
            Some(0),
            vec![
                "  ✓ sync: 1 remote: 1 ok".to_string(),
                last_run(&sb, "✓", "written just now"),
                "    ✓ fake: every 1m, last synced just now".to_string(),
            ]
        )
    );
    assert_eq!(listing(sb.dir.path()), before);
}

#[test]
fn a_failed_sync_is_a_warning_naming_dam_pull() {
    let _guard = support::guard("a_failed_sync_is_a_warning_naming_dam_pull");
    let sb = enabled(Remote::at("missing::").sync_every("1m"));
    assert_eq!(sb.output(&["sync", "run"]).status.code(), Some(1));
    let said = rows(&sb).1;
    assert_eq!(said.len(), 5, "{said:?}");
    assert!(
        said[3].starts_with("    ⚠ fake: last sync failed: "),
        "{said:?}"
    );
    assert_eq!(said[4], "      → run dam pull fake");
}

#[test]
fn the_json_names_the_last_run_file_and_when_it_was_written() {
    let _guard = support::guard("the_json_names_the_last_run_file");
    let sb = enabled(Remote::fake().sync_every("1m"));
    let (_, document) = sb.doctor_json(&["doctor", "sync"]);
    let path = sb.dir.path().join("state/dam/sync-last-run.txt");
    let check = &document["checks"][0];
    assert_eq!(check["last_run"]["path"], path.display().to_string());
    assert_eq!(check["last_run"]["written"], serde_json::Value::Null);

    assert!(
        sb.output(&["sync", "run", "--record-last-run"])
            .status
            .success()
    );
    let (_, document) = sb.doctor_json(&["doctor", "sync"]);
    assert!(document["checks"][0]["last_run"]["written"].is_string());
}

#[test]
fn a_remote_is_not_checked_on_a_store_this_dam_does_not_read() {
    let _guard = support::guard("a_remote_is_not_checked_on_a_store_this_dam_does_not_read");
    let sb = enabled(Remote::fake().sync_every("15m"));
    std::fs::write(sb.dir.path().join("dam.db"), "not a database ".repeat(40)).unwrap();
    assert_eq!(
        rows(&sb).1.last().map(String::as_str),
        Some("    · fake: not checked, the database was not read")
    );
}

#[test]
fn sync_is_not_checked_when_the_config_does_not_read() {
    let _guard = support::guard("sync_is_not_checked_when_the_config_does_not_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    assert_eq!(
        rows(&sb).1[0],
        "  · sync: not checked, the config did not read"
    );
}
