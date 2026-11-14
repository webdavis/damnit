use rusqlite::Connection;

pub(super) const VERSION: u32 = 12;

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

const V4: &str = r#"
UPDATE commit_changes SET op = 'create' WHERE op = 'update' AND before_json IS NULL;
UPDATE stage SET op = 'create' WHERE op = 'update' AND before_json IS NULL;
"#;

const V5: &str = r#"
UPDATE commit_changes SET before_json = (
    SELECT prior.after_json FROM commit_changes AS prior
    JOIN commits AS prior_commit ON prior_commit.id = prior.commit_id
    WHERE prior.oid = commit_changes.oid
      AND prior_commit.seq < (SELECT seq FROM commits WHERE id = commit_changes.commit_id)
    ORDER BY prior_commit.seq DESC LIMIT 1
) WHERE op = 'create' AND before_json IS NULL;
UPDATE commit_changes SET op = 'update' WHERE op = 'create' AND before_json IS NOT NULL;
ALTER TABLE objects DROP COLUMN kind;
"#;

const V6: &str = r#"
CREATE TABLE days (day TEXT PRIMARY KEY, json TEXT NOT NULL);
"#;

const V7: &str = r#"
CREATE TABLE items (hash TEXT PRIMARY KEY, token TEXT NOT NULL, key_id TEXT NOT NULL, channel TEXT NOT NULL, op TEXT NOT NULL, target TEXT NOT NULL, before_json TEXT, after_json TEXT, created TEXT NOT NULL, outcome TEXT NOT NULL, outcome_at TEXT);
CREATE TABLE stage_by_author (oid TEXT NOT NULL, op TEXT NOT NULL, before_json TEXT, after_json TEXT, author_token TEXT, author_key_id TEXT, author_channel TEXT);
INSERT INTO stage_by_author (oid, op, before_json, after_json) SELECT oid, op, before_json, after_json FROM stage;
DROP TABLE stage;
ALTER TABLE stage_by_author RENAME TO stage;
CREATE UNIQUE INDEX stage_one_change_per_author ON stage (oid, author_token IS NULL);
ALTER TABLE commit_changes ADD COLUMN author_token TEXT;
ALTER TABLE commit_changes ADD COLUMN author_key_id TEXT;
ALTER TABLE commit_changes ADD COLUMN author_channel TEXT;
"#;

const V8: &str = r#"
CREATE TABLE caller_ids (holder TEXT NOT NULL, token TEXT NOT NULL, session TEXT NOT NULL, id TEXT NOT NULL, oid TEXT NOT NULL, PRIMARY KEY (holder, token, session, id));
"#;

const V9: &str = r#"
ALTER TABLE stage ADD COLUMN details_json TEXT;
ALTER TABLE commit_changes ADD COLUMN details_json TEXT;
ALTER TABLE items ADD COLUMN details_json TEXT;
"#;

const V10: &str = r#"
UPDATE commit_changes SET author_token = NULL, author_key_id = NULL, author_channel = NULL, details_json = NULL
WHERE commit_id IN (
    SELECT c.id FROM commits AS c JOIN pushed AS p ON p.commit_id = c.id
    WHERE c.message = 'pull from ' || p.remote
);
"#;

const V11: &str = r#"
CREATE TABLE answers (remote TEXT NOT NULL, oid TEXT NOT NULL, count INTEGER NOT NULL, PRIMARY KEY (remote, oid));
"#;

const V12: &str = r#"
CREATE TABLE fields_in_doubt (remote TEXT NOT NULL, oid TEXT NOT NULL, field TEXT NOT NULL, PRIMARY KEY (remote, oid, field));
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
    if current < 4 {
        tx.execute_batch(V4)?;
    }
    if current < 5 {
        tx.execute_batch(V5)?;
    }
    if current < 6 {
        tx.execute_batch(V6)?;
    }
    if current < 7 {
        tx.execute_batch(V7)?;
    }
    if current < 8 {
        tx.execute_batch(V8)?;
    }
    if current < 9 {
        tx.execute_batch(V9)?;
    }
    if current < 10 {
        tx.execute_batch(V10)?;
    }
    if current < 11 {
        tx.execute_batch(V11)?;
    }
    if current < 12 {
        tx.execute_batch(V12)?;
    }
    tx.pragma_update(None, "user_version", VERSION)?;
    tx.commit()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod author_tests;

#[cfg(test)]
mod pulled_author_tests;

#[cfg(test)]
mod answer_count_tests;

#[cfg(test)]
mod doubt_tests;
