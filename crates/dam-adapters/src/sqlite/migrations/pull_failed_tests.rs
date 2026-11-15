use super::*;

fn a_store_at_version_holding_notices(version: u32, kinds: &[&str]) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    for (_, batch) in BATCHES.iter().filter(|(step, _)| *step <= version) {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", version).unwrap();
    for kind in kinds {
        conn.execute(
            "INSERT INTO notices (json) VALUES (?1)",
            [format!(r#"{{"kind":"{kind}","remote":"t","why":"w"}}"#)],
        )
        .unwrap();
    }
    conn
}

fn kinds_held(conn: &Connection) -> Vec<String> {
    let mut statement = conn
        .prepare("SELECT json_extract(json, '$.kind') FROM notices ORDER BY id")
        .unwrap();
    statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn count(conn: &Connection, sql: &str) -> u32 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn what_the_later_steps_left(conn: &Connection) -> (u32, u32, u32) {
    let version = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    let follow_ons = count(
        conn,
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'follow_ons'",
    );
    let pulled = count(
        conn,
        "SELECT count(*) FROM pragma_table_info('commits') WHERE name = 'pulled'",
    );
    (version, follow_ons, pulled)
}

#[test]
fn an_upgrade_drops_the_notices_a_read_left_when_its_pull_failed() {
    let mut conn = a_store_at_version_holding_notices(14, &["pull_failed", "push_failed"]);

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    assert_eq!(kinds_held(&conn), ["push_failed"]);
}

#[test]
fn a_store_at_each_earlier_version_runs_exactly_the_steps_it_lacks_in_order() {
    for version in [12, 13, 14] {
        let mut conn = a_store_at_version_holding_notices(version, &["pull_failed", "push_failed"]);

        migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

        assert_eq!(what_the_later_steps_left(&conn), (15, 1, 1), "{version}");
        assert_eq!(kinds_held(&conn), ["push_failed"], "{version}");
    }
}
