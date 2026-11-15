use super::*;

fn version_thirteen_store_with(commits: &[(&str, &str, bool)]) -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    for (_, batch) in &BATCHES[..13] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 13u32).unwrap();
    for (seq, (id, message, pushed)) in commits.iter().enumerate() {
        conn.execute_batch(&format!(
            "INSERT INTO commits (id, seq, message, at) VALUES ('{id}', {seq}, '{message}', '1970-01-01T00:00:00Z');"
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

fn pulled(conn: &Connection, id: &str) -> bool {
    conn.query_row("SELECT pulled FROM commits WHERE id = ?1", [id], |r| {
        r.get(0)
    })
    .unwrap()
}

#[test]
fn an_old_commit_a_pull_made_reads_as_pulled() {
    let conn = version_thirteen_store_with(&[("p", "pull from fake", true)]);
    assert!(pulled(&conn, "p"));
}

#[test]
fn an_old_commit_a_pull_could_not_have_made_reads_as_the_owners() {
    let conn = version_thirteen_store_with(&[
        ("c", "chore", true),
        ("n", "pull from fake", false),
        ("o", "pull from other", true),
    ]);
    for id in ["c", "n", "o"] {
        assert!(!pulled(&conn, id), "{id}");
    }
}
