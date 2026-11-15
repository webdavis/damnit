use super::*;

const TABLES_ADDED_SINCE_VERSION_TEN: [&str; 3] = ["answers", "attempts", "follow_ons"];

fn a_version_ten_store_holding_a_remote_id() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6, V7, V8, V9, V10] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 10u32).unwrap();
    conn.execute(
        "INSERT INTO remote_ids (remote, oid, remote_id) VALUES ('todoist', 'o', 'r1')",
        [],
    )
    .unwrap();
    conn
}

fn rows_in(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap_or_else(|e| panic!("{table}: {e}"))
}

fn schema_and_version(conn: &Connection) -> (Vec<String>, u32) {
    let mut statement = conn
        .prepare("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
        .unwrap();
    let schema = statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let version = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    (schema, version)
}

#[test]
fn an_upgraded_store_gains_empty_tables_and_keeps_its_rows() {
    let mut conn = a_version_ten_store_holding_a_remote_id();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    for table in TABLES_ADDED_SINCE_VERSION_TEN {
        assert_eq!(rows_in(&conn, table), 0, "{table}");
    }
    let kept: String = conn
        .query_row(
            "SELECT remote_id FROM remote_ids WHERE oid = 'o'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(kept, "r1");
}

#[test]
fn a_second_upgrade_changes_nothing() {
    let mut conn = a_version_ten_store_holding_a_remote_id();
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    let first = schema_and_version(&conn);

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    assert_eq!(schema_and_version(&conn), first);
    assert_eq!(rows_in(&conn, "remote_ids"), 1);
}
