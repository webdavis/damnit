mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use support::config_files::Remote;
use support::doctor::listing;
use support::sandbox::Sandbox;

const A_HELPER_HANGS_UNTIL_ITS_ONE_SECOND_DEADLINE: &str =
    "the helper hangs until its one second deadline";

fn commit_task(sb: &Sandbox, subject: &str) {
    sb.new_object(&[subject]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", subject]).0);
}

fn hanging_helper_that_marks_its_start(sb: &Sandbox) -> std::path::PathBuf {
    let started = sb.dir.path().join("helper-started");
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    let script = format!(
        "#!/bin/sh\ntouch '{}'\nwhile :; do sleep 0.05; done\n",
        started.display()
    );
    std::fs::write(&helper, script).unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    started
}

fn doctor_until_the_helper_starts(sb: &Sandbox, started: &Path) -> Child {
    let doctor = sb
        .command(&["doctor", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let give_up_at = Instant::now() + Duration::from_secs(10);
    while !started.exists() {
        assert!(Instant::now() < give_up_at, "the helper never started");
        std::thread::sleep(Duration::from_millis(10));
    }
    doctor
}

fn pushes_row(doctor: Child) -> serde_json::Value {
    let out = doctor.wait_with_output().unwrap();
    assert!(out.stderr.is_empty(), "{out:?}");
    let document: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let checks = document["checks"].as_array().unwrap();
    let pushes = checks.iter().find(|c| c["name"] == "pushes").unwrap();
    pushes["rows"][0].clone()
}

fn no_side_file(folder: &Path) -> bool {
    let names = listing(folder);
    !names.contains("dam.db-wal") && !names.contains("dam.db-shm")
}

#[test]
fn what_is_owed_is_read_before_a_helper_starts_while_another_dam_writes() {
    let _guard =
        support::guard("what_is_owed_is_read_before_a_helper_starts_while_another_dam_writes")
            .allow_slow(A_HELPER_HANGS_UNTIL_ITS_ONE_SECOND_DEADLINE);
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("1s"));
    commit_task(&sb, "buy oat milk");
    commit_task(&sb, "call Ada");
    let started = hanging_helper_that_marks_its_start(&sb);
    let doctor = doctor_until_the_helper_starts(&sb, &started);
    commit_task(&sb, "water the plants");
    let row = pushes_row(doctor);
    assert_eq!(row["detail"], "2 changes still owed", "{row}");
    assert!(no_side_file(sb.dir.path()), "{:?}", listing(sb.dir.path()));
}

#[test]
fn a_store_another_dam_holds_open_is_let_go_before_a_helper_starts() {
    let _guard = support::guard("a_store_another_dam_holds_open_is_let_go_before_a_helper_starts")
        .allow_slow(A_HELPER_HANGS_UNTIL_ITS_ONE_SECOND_DEADLINE);
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("1s"));
    commit_task(&sb, "buy oat milk");
    let holder = rusqlite::Connection::open(sb.dir.path().join("dam.db")).unwrap();
    let _: i64 = holder
        .query_row("select count(*) from sqlite_master", [], |r| r.get(0))
        .unwrap();
    assert!(!no_side_file(sb.dir.path()), "{:?}", listing(sb.dir.path()));
    let started = hanging_helper_that_marks_its_start(&sb);
    let doctor = doctor_until_the_helper_starts(&sb, &started);
    drop(holder);
    assert!(no_side_file(sb.dir.path()), "{:?}", listing(sb.dir.path()));
    let row = pushes_row(doctor);
    assert_eq!(row["detail"], "1 change still owed", "{row}");
}
