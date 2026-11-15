use crate::support;

use std::path::PathBuf;

use support::doctor::listing;
use support::sandbox::Sandbox;

const SUPPORTED: u32 = 15;

fn store(sb: &Sandbox) -> PathBuf {
    sb.dir.path().join("dam.db")
}

fn set_version(sb: &Sandbox, version: u32) {
    let conn = rusqlite::Connection::open(store(sb)).unwrap();
    conn.pragma_update(None, "user_version", version).unwrap();
}

fn row(sb: &Sandbox) -> (Option<i32>, String) {
    let answer = sb.doctor(&["doctor", "database"]);
    let row = answer.out.lines().nth(3).unwrap().to_string();
    (
        answer.code,
        format!("{row}\n{}", answer.out.lines().nth(4).unwrap()),
    )
}

#[test]
fn no_store_yet_is_well_and_stays_absent() {
    let _guard = support::guard("no_store_yet_is_well_and_stays_absent");
    let sb = Sandbox::new();
    let before = listing(sb.dir.path());
    let (code, row) = row(&sb);
    assert_eq!(code, Some(0));
    assert!(row.starts_with("  ✓ database: none yet\n"), "{row}");
    assert_eq!(listing(sb.dir.path()), before);
}

#[test]
fn a_current_store_names_its_version_and_gains_no_side_file() {
    let _guard = support::guard("a_current_store_names_its_version_and_gains_no_side_file");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    let before = listing(sb.dir.path());
    assert!(
        before.contains("dam.db") && !before.contains("dam.db-wal"),
        "{before:?}"
    );
    let (code, row) = row(&sb);
    assert_eq!(code, Some(0));
    assert!(
        row.starts_with(&format!("  ✓ database: version {SUPPORTED}\n")),
        "{row}"
    );
    assert_eq!(listing(sb.dir.path()), before);
}

#[test]
fn an_older_store_is_a_warning_the_next_command_mends() {
    let _guard = support::guard("an_older_store_is_a_warning_the_next_command_mends");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    set_version(&sb, SUPPORTED - 1);
    let (code, row) = row(&sb);
    assert_eq!(code, Some(0));
    let said = format!(
        "  ⚠ database: version {}, the next dam command upgrades it to {SUPPORTED}\n",
        SUPPORTED - 1
    );
    assert!(row.starts_with(&said), "{row}");
}

#[test]
fn a_newer_store_is_an_issue_with_advice() {
    let _guard = support::guard("a_newer_store_is_an_issue_with_advice");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    set_version(&sb, SUPPORTED + 1);
    let (code, row) = row(&sb);
    assert_eq!(code, Some(1));
    assert_eq!(
        row,
        format!(
            "  ✗ database: version {}, newer than the {SUPPORTED} this dam reads\n    → upgrade dam",
            SUPPORTED + 1
        )
    );
}

#[test]
fn a_file_that_is_no_store_is_an_issue() {
    let _guard = support::guard("a_file_that_is_no_store_is_an_issue");
    let sb = Sandbox::new();
    std::fs::write(store(&sb), "not a database, only text ".repeat(40)).unwrap();
    let (code, row) = row(&sb);
    assert_eq!(code, Some(1));
    let said = format!("  ✗ database: {} does not open: ", store(&sb).display());
    assert!(row.starts_with(&said), "{row}");
    assert!(
        row.ends_with("\n    → restore it from a backup, or point DAM_STORE at another file"),
        "{row}"
    );
}

fn left_with_only_its_wal(sb: &Sandbox, file_version: u32, wal_version: u32) {
    sb.new_object(&["buy milk"]);
    set_version(sb, file_version);
    let wal = sb.dir.path().join("dam.db-wal");
    let kept = tempfile::tempdir().unwrap();
    let conn = rusqlite::Connection::open(store(sb)).unwrap();
    conn.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    conn.pragma_update(None, "user_version", wal_version)
        .unwrap();
    std::fs::copy(store(sb), kept.path().join("dam.db")).unwrap();
    std::fs::copy(&wal, kept.path().join("dam.db-wal")).unwrap();
    drop(conn);
    std::fs::copy(kept.path().join("dam.db"), store(sb)).unwrap();
    std::fs::copy(kept.path().join("dam.db-wal"), &wal).unwrap();
    let left = listing(sb.dir.path());
    assert!(
        left.contains("dam.db-wal") && !left.contains("dam.db-shm"),
        "{left:?}"
    );
}

const ONLY_THE_WAL: &str = "  ⚠ database: not read, dam.db-wal is left beside it, so the file alone may read wrong\n    → run dam status, which opens the store and settles what is left, then run dam doctor";

#[test]
fn a_store_left_with_only_its_wal_is_a_warning_and_names_no_version() {
    let _guard = support::guard("a_store_left_with_only_its_wal_is_a_warning_and_names_no_version");
    for (file_version, wal_version) in [(SUPPORTED, SUPPORTED + 1), (SUPPORTED - 1, SUPPORTED)] {
        let sb = Sandbox::new();
        left_with_only_its_wal(&sb, file_version, wal_version);
        let before = listing(sb.dir.path());
        let (code, row) = row(&sb);
        assert_eq!(code, Some(0));
        assert_eq!(row, ONLY_THE_WAL);
        assert_eq!(listing(sb.dir.path()), before);
    }
}

#[test]
fn the_command_the_warning_names_brings_the_store_up_to_date() {
    let _guard = support::guard("the_command_the_warning_names_brings_the_store_up_to_date");
    let sb = Sandbox::new();
    left_with_only_its_wal(&sb, SUPPORTED - 1, SUPPORTED);
    let (ok, _, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    let (code, row) = row(&sb);
    assert_eq!(code, Some(0));
    assert!(
        row.starts_with(&format!("  ✓ database: version {SUPPORTED}\n")),
        "{row}"
    );
    let after = listing(sb.dir.path());
    assert!(
        !after.contains("dam.db-wal") && !after.contains("dam.db-shm"),
        "{after:?}"
    );
}
