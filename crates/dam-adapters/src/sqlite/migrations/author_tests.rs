use super::*;

#[test]
fn a_version_six_change_gains_an_empty_author_that_reads_as_the_owner() {
    let mut conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 6u32).unwrap();
    conn.execute_batch(
        "INSERT INTO commits (id, seq, message, at) VALUES ('c', 1, 'm', '1970-01-01T00:00:00Z');
         INSERT INTO commit_changes (commit_id, ord, oid, op, before_json, after_json) VALUES ('c', 0, 'o', 'create', NULL, '{}');
         INSERT INTO stage (oid, op, before_json, after_json) VALUES ('o', 'create', NULL, '{}');",
    )
    .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    for table in ["commit_changes", "stage"] {
        let author: (Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                &format!("SELECT author_token, author_key_id, author_channel FROM {table}"),
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(author, (None, None, None), "{table}");
    }
}
