use rusqlite::Connection;

pub(super) const VERSION: u32 = 3;

const V1: &str = r#"
CREATE TABLE objects (oid TEXT PRIMARY KEY, kind TEXT NOT NULL, path TEXT NOT NULL, json TEXT NOT NULL);
CREATE INDEX objects_path ON objects(path);
CREATE TABLE committed (oid TEXT PRIMARY KEY, json TEXT NOT NULL);
CREATE TABLE stage (oid TEXT PRIMARY KEY, op TEXT NOT NULL, before_json TEXT, after_json TEXT);
CREATE TABLE commits (id TEXT PRIMARY KEY, seq INTEGER NOT NULL UNIQUE, message TEXT NOT NULL, at TEXT NOT NULL);
CREATE TABLE commit_changes (commit_id TEXT NOT NULL, ord INTEGER NOT NULL, oid TEXT NOT NULL, op TEXT NOT NULL, before_json TEXT, after_json TEXT, PRIMARY KEY (commit_id, ord));
CREATE TABLE pushed (remote TEXT NOT NULL, commit_id TEXT NOT NULL, PRIMARY KEY (remote, commit_id));
CREATE TABLE remote_ids (remote TEXT NOT NULL, oid TEXT NOT NULL, remote_id TEXT NOT NULL, PRIMARY KEY (remote, oid), UNIQUE (remote, remote_id));
CREATE TABLE snapshots (remote TEXT NOT NULL, oid TEXT NOT NULL, json TEXT NOT NULL, PRIMARY KEY (remote, oid));
CREATE TABLE sync_tokens (remote TEXT PRIMARY KEY, token TEXT);
CREATE TABLE conflicts (oid TEXT PRIMARY KEY, remote TEXT NOT NULL, ours_json TEXT NOT NULL, theirs_json TEXT NOT NULL);
CREATE TABLE notices (id INTEGER PRIMARY KEY AUTOINCREMENT, json TEXT NOT NULL);
CREATE TABLE push_retries (remote TEXT NOT NULL, oid TEXT NOT NULL, PRIMARY KEY (remote, oid));
CREATE TABLE pulls (remote TEXT PRIMARY KEY, at TEXT NOT NULL);
"#;

const V2: &str = r#"
CREATE TABLE pushes (remote TEXT PRIMARY KEY, at TEXT NOT NULL);
"#;

const V3: &str = r#"
CREATE TABLE accepted_kinds (remote TEXT PRIMARY KEY, kinds TEXT NOT NULL);
"#;

pub(super) fn migrate_above_user_version_in_one_transaction(
    conn: &mut Connection,
) -> Result<(), rusqlite::Error> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if current >= VERSION {
        return Ok(());
    }
    let tx = conn.transaction()?;
    if current < 1 {
        tx.execute_batch(V1)?;
    }
    if current < 2 {
        tx.execute_batch(V2)?;
    }
    if current < 3 {
        tx.execute_batch(V3)?;
    }
    tx.pragma_update(None, "user_version", VERSION)?;
    tx.commit()
}

#[cfg(test)]
mod tests;
