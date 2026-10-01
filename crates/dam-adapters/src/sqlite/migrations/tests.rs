use super::*;

#[test]
fn a_version_one_store_migrates_up_and_keeps_its_rows() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(V1).unwrap();
    conn.pragma_update(None, "user_version", 1u32).unwrap();
    conn.execute(
        "INSERT INTO pulls (remote, at) VALUES ('todoist', '1970-01-01T00:00:00Z')",
        [],
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    let version: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, VERSION);
    conn.execute(
        "INSERT INTO pushes (remote, at) VALUES ('todoist', '1970-01-01T00:01:00Z')",
        [],
    )
    .unwrap();
    let pulled: String = conn
        .query_row("SELECT at FROM pulls WHERE remote = 'todoist'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(pulled, "1970-01-01T00:00:00Z");
}

#[test]
fn migrating_a_current_store_again_changes_nothing() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, VERSION);
}

#[test]
fn a_version_two_store_gains_the_accepted_kinds_table() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(V1).unwrap();
    conn.execute_batch(V2).unwrap();
    conn.pragma_update(None, "user_version", 2u32).unwrap();
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    conn.execute(
        "INSERT INTO accepted_kinds (remote, kinds) VALUES ('gcal', '')",
        [],
    )
    .unwrap();
}
