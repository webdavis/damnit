use super::*;

#[test]
fn a_version_eleven_store_gains_fields_in_doubt_and_keeps_its_count_of_answers() {
    let mut conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 11u32).unwrap();
    conn.execute(
        "INSERT INTO answers (remote, oid, count) VALUES ('todoist', 'o', 2)",
        [],
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    conn.execute(
        "INSERT INTO fields_in_doubt (remote, oid, field) VALUES ('todoist', 'o', 'subject')",
        [],
    )
    .unwrap();
    let kept: i64 = conn
        .query_row("SELECT count FROM answers WHERE oid = 'o'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(kept, 2);
}
