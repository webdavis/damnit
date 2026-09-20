use dam_application::{CommitRepository, RemoteName, StoreError};
use dam_domain::{Change, CommitId, CommitRecord, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::codec::{object_to_json, op_to_text, read_changes};
use super::objects::sql;

impl SqliteStore {
    pub(super) fn commit_record(&self, record: &CommitRecord) -> Result<(), StoreError> {
        self.in_savepoint("dam_commit", || self.write_commit(record))
    }

    fn write_commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        let seq: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM commits", [], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        self.conn
            .execute(
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
            let before = change.before.as_ref().map(object_to_json).transpose()?;
            let after = change.after.as_ref().map(object_to_json).transpose()?;
            self.conn.execute(
                "INSERT INTO commit_changes (commit_id, ord, oid, op, before_json, after_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    record.id.as_str(),
                    ord as i64,
                    change.oid.as_str(),
                    op_to_text(change.op),
                    before,
                    after
                ],
            )
            .map_err(sql)?;
            match &change.after {
                Some(after) => self
                    .conn
                    .execute(
                        "INSERT INTO committed (oid, json) VALUES (?1, ?2)
                         ON CONFLICT(oid) DO UPDATE SET json = excluded.json",
                        params![change.oid.as_str(), object_to_json(after)?],
                    )
                    .map_err(sql)?,
                None => self
                    .conn
                    .execute(
                        "DELETE FROM committed WHERE oid = ?1",
                        params![change.oid.as_str()],
                    )
                    .map_err(sql)?,
            };
            self.conn
                .execute(
                    "DELETE FROM stage WHERE oid = ?1",
                    params![change.oid.as_str()],
                )
                .map_err(sql)?;
        }
        Ok(())
    }

    pub(super) fn commit_log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, message, at FROM commits ORDER BY seq DESC")
            .map_err(sql)?;
        let heads: Vec<(String, String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(sql)?
            .collect::<Result<_, _>>()
            .map_err(sql)?;
        heads
            .into_iter()
            .map(|(id, message, at)| self.record(id, message, at))
            .collect()
    }

    /// One commit head plus the changes it carries.
    fn record(&self, id: String, message: String, at: String) -> Result<CommitRecord, StoreError> {
        Ok(CommitRecord {
            id: CommitId::parse(&id).map_err(|e| StoreError::Failed(e.to_string()))?,
            message,
            at: at
                .parse()
                .map_err(|e: jiff::Error| StoreError::Failed(e.to_string()))?,
            changes: self.commit_changes(&id)?,
        })
    }

    fn commit_changes(&self, commit_id: &str) -> Result<Vec<Change>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT oid, op, before_json, after_json FROM commit_changes WHERE commit_id = ?1 ORDER BY ord",
            )
            .map_err(sql)?;
        read_changes(&mut stmt, params![commit_id])
    }

    /// Every commit this remote has not been told about, oldest first. The
    /// filter is the query rather than a scan of the decoded log, so the work
    /// is one pass over the unpushed commits and not over all of history.
    pub(super) fn unpushed_commits(
        &self,
        remote: &RemoteName,
    ) -> Result<Vec<CommitRecord>, StoreError> {
        let heads: Vec<(String, String, String)> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT id, message, at FROM commits WHERE NOT EXISTS \
                     (SELECT 1 FROM pushed WHERE pushed.remote = ?1 AND pushed.commit_id = commits.id) \
                     ORDER BY seq",
                )
                .map_err(sql)?;
            stmt.query_map(params![remote.0], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(sql)?
                .collect::<Result<_, _>>()
                .map_err(sql)?
        };
        heads
            .into_iter()
            .map(|(id, message, at)| self.record(id, message, at))
            .collect()
    }

    pub(super) fn mark_commit_pushed(
        &self,
        remote: &RemoteName,
        id: &CommitId,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO pushed (remote, commit_id) VALUES (?1, ?2)",
                params![remote.0, id.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

impl CommitRepository for SqliteStore {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        self.commit_record(record)
    }
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        self.commit_log()
    }
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError> {
        self.unpushed_commits(remote)
    }
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError> {
        self.mark_commit_pushed(remote, id)
    }
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        self.retries_of(remote)
    }
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.set_retries(remote, oids)
    }
}
