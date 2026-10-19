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

#[test]
fn a_version_three_update_with_no_before_reads_as_the_create_it_is() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(V1).unwrap();
    conn.execute_batch(V2).unwrap();
    conn.execute_batch(V3).unwrap();
    conn.pragma_update(None, "user_version", 3u32).unwrap();
    conn.execute_batch(
        "INSERT INTO commit_changes VALUES ('c', 0, 'o', 'update', NULL, '{}');
         INSERT INTO commit_changes VALUES ('c', 1, 'p', 'update', '{}', '{}');
         INSERT INTO stage VALUES ('o', 'update', NULL, '{}');",
    )
    .unwrap();
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    let ops = |table: &str| -> Vec<String> {
        conn.prepare(&format!("SELECT op FROM {table} ORDER BY oid"))
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    assert_eq!(ops("commit_changes"), vec!["create", "update"]);
    assert_eq!(ops("stage"), vec!["create"]);
}

fn store_at_version_three() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(V1).unwrap();
    conn.execute_batch(V2).unwrap();
    conn.execute_batch(V3).unwrap();
    conn.pragma_update(None, "user_version", 3u32).unwrap();
    conn
}

fn change_rows(conn: &Connection) -> Vec<(String, String, Option<String>)> {
    conn.prepare("SELECT oid, op, before_json FROM commit_changes ORDER BY commit_id, ord")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn an_old_pull_update_with_no_before_takes_its_before_from_the_prior_commit() {
    let mut conn = store_at_version_three();
    conn.execute_batch(
        "INSERT INTO commits VALUES ('c1', 1, 'add', '1970-01-01T00:00:00Z');
         INSERT INTO commits VALUES ('c2', 2, 'pull', '1970-01-01T00:01:00Z');
         INSERT INTO commits VALUES ('c3', 3, 'rm', '1970-01-01T00:02:00Z');
         INSERT INTO commits VALUES ('c4', 4, 'pull', '1970-01-01T00:03:00Z');
         INSERT INTO commit_changes VALUES ('c1', 0, 'o', 'create', NULL, '\"first\"');
         INSERT INTO commit_changes VALUES ('c1', 1, 'g', 'create', NULL, '\"gone\"');
         INSERT INTO commit_changes VALUES ('c2', 0, 'o', 'update', NULL, '\"second\"');
         INSERT INTO commit_changes VALUES ('c2', 1, 'p', 'update', NULL, '\"new\"');
         INSERT INTO commit_changes VALUES ('c3', 0, 'g', 'delete', '\"gone\"', NULL);
         INSERT INTO commit_changes VALUES ('c4', 0, 'g', 'update', NULL, '\"back\"');
         INSERT INTO commit_changes VALUES ('c4', 1, 'o', 'update', NULL, '\"third\"');",
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    let row = |oid: &str, op: &str, before: Option<&str>| {
        (oid.to_string(), op.to_string(), before.map(str::to_string))
    };
    assert_eq!(
        change_rows(&conn),
        vec![
            row("o", "create", None),
            row("g", "create", None),
            row("o", "update", Some("\"first\"")),
            row("p", "create", None),
            row("g", "delete", Some("\"gone\"")),
            row("g", "create", None),
            row("o", "update", Some("\"second\"")),
        ]
    );
}

#[test]
fn the_unread_kind_column_leaves_objects_and_its_rows_stay() {
    let mut conn = store_at_version_three();
    conn.execute(
        "INSERT INTO objects (oid, kind, path, json) VALUES ('o', 'task', 'work/', '{}')",
        [],
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    let columns: Vec<String> = conn
        .prepare("SELECT name FROM pragma_table_info('objects') ORDER BY cid")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(columns, vec!["oid", "path", "json"]);
    let path: String = conn
        .query_row("SELECT path FROM objects WHERE oid = 'o'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(path, "work/");
}
