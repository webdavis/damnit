use super::*;

#[test]
fn a_version_ten_store_gains_a_count_of_answers_and_keeps_its_rows() {
    let mut conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6, V7, V8, V9, V10] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 10u32).unwrap();
    conn.execute(
        "INSERT INTO remote_ids (remote, oid, remote_id) VALUES ('todoist', 'o', 'r1')",
        [],
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    conn.execute(
        "INSERT INTO answers (remote, oid, count) VALUES ('todoist', 'o', 1)",
        [],
    )
    .unwrap();
    let kept: String = conn
        .query_row(
            "SELECT remote_id FROM remote_ids WHERE oid = 'o'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(kept, "r1");
}
