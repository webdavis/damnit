pub(super) const V1: &str = r#"
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

pub(super) const V2: &str = r#"
CREATE TABLE pushes (remote TEXT PRIMARY KEY, at TEXT NOT NULL);
"#;

pub(super) const V3: &str = r#"
CREATE TABLE accepted_kinds (remote TEXT PRIMARY KEY, kinds TEXT NOT NULL);
"#;

pub(super) const V4: &str = r#"
UPDATE commit_changes SET op = 'create' WHERE op = 'update' AND before_json IS NULL;
UPDATE stage SET op = 'create' WHERE op = 'update' AND before_json IS NULL;
"#;

pub(super) const V5: &str = r#"
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

pub(super) const V6: &str = r#"
CREATE TABLE days (day TEXT PRIMARY KEY, json TEXT NOT NULL);
"#;

pub(super) const V7: &str = r#"
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

pub(super) const V8: &str = r#"
CREATE TABLE caller_ids (holder TEXT NOT NULL, token TEXT NOT NULL, session TEXT NOT NULL, id TEXT NOT NULL, oid TEXT NOT NULL, PRIMARY KEY (holder, token, session, id));
"#;

pub(super) const V9: &str = r#"
ALTER TABLE stage ADD COLUMN details_json TEXT;
ALTER TABLE commit_changes ADD COLUMN details_json TEXT;
ALTER TABLE items ADD COLUMN details_json TEXT;
"#;

pub(super) const V10: &str = r#"
UPDATE commit_changes SET author_token = NULL, author_key_id = NULL, author_channel = NULL, details_json = NULL
WHERE commit_id IN (
    SELECT c.id FROM commits AS c JOIN pushed AS p ON p.commit_id = c.id
    WHERE c.message = 'pull from ' || p.remote
);
"#;

pub(super) const V11: &str = r#"
CREATE TABLE answers (remote TEXT NOT NULL, oid TEXT NOT NULL, count INTEGER NOT NULL, PRIMARY KEY (remote, oid));
"#;

pub(super) const V12: &str = r#"
CREATE TABLE attempts (remote TEXT NOT NULL, oid TEXT NOT NULL, position INTEGER NOT NULL, key TEXT NOT NULL, op TEXT NOT NULL, container_remote_id TEXT, declared TEXT NOT NULL, PRIMARY KEY (remote, oid, position));
"#;

pub(super) const V13: &str = r#"
CREATE TABLE follow_ons (hash TEXT NOT NULL, position INTEGER NOT NULL, moment TEXT NOT NULL, working_json TEXT NOT NULL, op TEXT, before_json TEXT, after_json TEXT, author_token TEXT, author_key_id TEXT, author_channel TEXT, details_json TEXT, PRIMARY KEY (hash, position, moment));
"#;

pub(super) const V14: &str = r#"
ALTER TABLE commits ADD COLUMN pulled INTEGER NOT NULL DEFAULT 0;
UPDATE commits SET pulled = 1 WHERE id IN (
    SELECT c.id FROM commits AS c JOIN pushed AS p ON p.commit_id = c.id
    WHERE c.message = 'pull from ' || p.remote
);
"#;

pub(super) const V15: &str = r#"
DELETE FROM notices WHERE json_extract(json, '$.kind') = 'pull_failed';
"#;

pub(super) const BATCHES: [(u32, &str); 15] = [
    (1, V1),
    (2, V2),
    (3, V3),
    (4, V4),
    (5, V5),
    (6, V6),
    (7, V7),
    (8, V8),
    (9, V9),
    (10, V10),
    (11, V11),
    (12, V12),
    (13, V13),
    (14, V14),
    (15, V15),
];
