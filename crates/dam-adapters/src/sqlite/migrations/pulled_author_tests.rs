use super::*;

const TOKEN: &str = "'agent', '0123456789abcdef', 'cli', '{}'";

fn version_nine_store_with(commits: &[(&str, &str, bool)]) -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6, V7, V8, V9] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 9u32).unwrap();
    for (seq, (id, message, pushed)) in commits.iter().enumerate() {
        conn.execute_batch(&format!(
            "INSERT INTO commits (id, seq, message, at) VALUES ('{id}', {seq}, '{message}', '1970-01-01T00:00:00Z');
             INSERT INTO commit_changes (commit_id, ord, oid, op, before_json, after_json, author_token, author_key_id, author_channel, details_json)
             VALUES ('{id}', 0, '{id}', 'create', NULL, '{{}}', {TOKEN});"
        ))
        .unwrap();
        if *pushed {
            conn.execute_batch(&format!(
                "INSERT INTO pushed (remote, commit_id) VALUES ('fake', '{id}');"
            ))
            .unwrap();
        }
    }
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    conn
}

fn author_of(conn: &Connection, id: &str) -> (Option<String>, Option<String>) {
    conn.query_row(
        "SELECT author_token, details_json FROM commit_changes WHERE commit_id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .unwrap()
}

#[test]
fn a_change_a_pull_landed_under_a_token_becomes_the_owners() {
    let conn = version_nine_store_with(&[("p", "pull from fake", true)]);
    assert_eq!(author_of(&conn, "p"), (None, None));
}

#[test]
fn a_tokens_own_commits_keep_their_author_even_once_pushed() {
    let conn = version_nine_store_with(&[
        ("c", "chore", true),
        ("n", "pull from fake", false),
        ("o", "pull from other", true),
    ]);
    for id in ["c", "n", "o"] {
        assert_eq!(author_of(&conn, id).0.as_deref(), Some("agent"), "{id}");
    }
}
