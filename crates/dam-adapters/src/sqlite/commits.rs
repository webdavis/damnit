use dam_application::{CommitRepository, RemoteName, StoreError};
use dam_domain::{Change, CommitId, CommitRecord, Oid};
use rusqlite::{Connection, params};

use super::SqliteStore;
use super::codec::{object_to_json, op_to_text, read_changes};
use super::objects::sql;

impl CommitRepository for SqliteStore {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        self.in_savepoint("dam_commit", || write_commit(&self.conn, record))
    }

    fn log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        records(
            &self.conn,
            "SELECT id, message, at FROM commits ORDER BY seq DESC",
            [],
        )
    }

    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError> {
        records(
            &self.conn,
            "SELECT id, message, at FROM commits WHERE NOT EXISTS \
             (SELECT 1 FROM pushed WHERE pushed.remote = ?1 AND pushed.commit_id = commits.id) \
             ORDER BY seq",
            params![remote.0],
        )
    }

    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO pushed (remote, commit_id) VALUES (?1, ?2)",
                params![remote.0, id.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT oid FROM push_retries WHERE remote = ?1 ORDER BY oid")
            .map_err(sql)?;
        let rows = stmt
            .query_map(params![remote.0], |r| r.get::<_, String>(0))
            .map_err(sql)?;
        rows.map(|r| {
            r.map_err(sql)
                .and_then(|o| Oid::parse(&o).map_err(|e| StoreError::Failed(e.to_string())))
        })
        .collect()
    }

    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.in_savepoint("dam_retries", || {
            self.conn
                .execute(
                    "DELETE FROM push_retries WHERE remote = ?1",
                    params![remote.0],
                )
                .map_err(sql)?;
            for oid in oids {
                self.conn
                    .execute(
                        "INSERT INTO push_retries (remote, oid) VALUES (?1, ?2)",
                        params![remote.0, oid.as_str()],
                    )
                    .map_err(sql)?;
            }
            Ok(())
        })
    }
}

fn write_commit(conn: &Connection, record: &CommitRecord) -> Result<(), StoreError> {
    let seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM commits", [], |r| {
            r.get(0)
        })
        .map_err(sql)?;
    conn.execute(
        "INSERT INTO commits (id, seq, message, at) VALUES (?1, ?2, ?3, ?4)",
        params![
            record.id.as_str(),
            seq,
            record.message,
            record.at.to_string()
        ],
    )
    .map_err(sql)?;
    for (ord, change) in record.changes.iter().enumerate() {
        write_change(conn, record, ord, change)?;
    }
    Ok(())
}

fn write_change(
    conn: &Connection,
    record: &CommitRecord,
    ord: usize,
    change: &Change,
) -> Result<(), StoreError> {
    let oid = change.oid().as_str();
    let before = change.before().map(object_to_json).transpose()?;
    let after = change.after().map(object_to_json).transpose()?;
    conn.execute(
        "INSERT INTO commit_changes (commit_id, ord, oid, op, before_json, after_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![record.id.as_str(), ord as i64, oid, op_to_text(change.op()), before, after],
    )
    .map_err(sql)?;
    match after {
        Some(after) => conn.execute(
            "INSERT INTO committed (oid, json) VALUES (?1, ?2)
             ON CONFLICT(oid) DO UPDATE SET json = excluded.json",
            params![oid, after],
        ),
        None => conn.execute("DELETE FROM committed WHERE oid = ?1", params![oid]),
    }
    .map_err(sql)?;
    conn.execute("DELETE FROM stage WHERE oid = ?1", params![oid])
        .map(|_| ())
        .map_err(sql)
}

fn records(
    conn: &Connection,
    heads_query: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<CommitRecord>, StoreError> {
    let heads: Vec<(String, String, String)> = conn
        .prepare(heads_query)
        .map_err(sql)?
        .query_map(params, |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(sql)?
        .collect::<Result<_, _>>()
        .map_err(sql)?;
    heads
        .into_iter()
        .map(|(id, message, at)| {
            let mut stmt = conn
                .prepare(
                    "SELECT op, before_json, after_json FROM commit_changes WHERE commit_id = ?1 ORDER BY ord",
                )
                .map_err(sql)?;
            Ok(CommitRecord {
                id: CommitId::parse(&id).map_err(|e| StoreError::Failed(e.to_string()))?,
                message,
                at: at
                    .parse()
                    .map_err(|e: jiff::Error| StoreError::Failed(e.to_string()))?,
                changes: read_changes(&mut stmt, params![id])?,
            })
        })
        .collect()
}
