use dam_application::{ObjectStore, RemoteName, StoreError};
use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Path, coalesce};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::codec::{change_from_row, object_to_json, op_to_text, read_changes};
use super::objects::sql;

impl SqliteStore {
    fn staged_change(&self, oid: &Oid) -> Result<Option<Change>, StoreError> {
        self.conn
            .query_row(
                "SELECT op, before_json, after_json FROM stage WHERE oid = ?1",
                params![oid.as_str()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(sql)?
            .map(|(op, b, a)| change_from_row(oid.as_str(), &op, b, a))
            .transpose()
    }

    fn write_stage(&self, change: &Change) -> Result<(), StoreError> {
        let before = change.before.as_ref().map(object_to_json).transpose()?;
        let after = change.after.as_ref().map(object_to_json).transpose()?;
        self.conn
            .execute(
                "INSERT INTO stage (oid, op, before_json, after_json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(oid) DO UPDATE SET op = excluded.op, before_json = excluded.before_json, after_json = excluded.after_json",
                params![change.oid.as_str(), op_to_text(change.op), before, after],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

impl ObjectStore for SqliteStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.get_object(oid)
    }
    fn all(&self) -> Result<Vec<Object>, StoreError> {
        self.all_objects()
    }
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError> {
        self.children(path)
    }
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        self.dependents(oid)
    }
    fn put(&self, object: &Object) -> Result<(), StoreError> {
        self.put_object(object)
    }
    fn delete(&self, oid: &Oid) -> Result<(), StoreError> {
        self.delete_object(oid)
    }
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.committed_object(oid)
    }

    fn stage(&self, change: Change) -> Result<(), StoreError> {
        let merged = match self.staged_change(&change.oid)? {
            Some(existing) => coalesce(existing, change.clone()),
            None => Some(change.clone()),
        };
        match merged {
            Some(c) => self.write_stage(&c),
            None => self.unstage(&change.oid),
        }
    }

    fn unstage(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM stage WHERE oid = ?1", params![oid.as_str()])
            .map(|_| ())
            .map_err(sql)
    }

    fn unstage_all(&self) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM stage", [])
            .map(|_| ())
            .map_err(sql)
    }

    fn staged(&self) -> Result<Vec<Change>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT oid, op, before_json, after_json FROM stage ORDER BY oid")
            .map_err(sql)?;
        read_changes(&mut stmt, [])
    }

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
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        self.remote_id_of(remote, oid)
    }
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        self.oid_of_remote_id(remote, remote_id)
    }
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.set_remote_id(remote, oid, remote_id)
    }
    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        self.snapshot_of(remote, oid)
    }
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.set_snapshot(remote, object)
    }
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.clear_mapping(remote, oid)
    }
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        self.token_of(remote)
    }
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        self.set_token(remote, token)
    }
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        self.record_conflict(remote, oid, theirs)
    }
    fn conflicts(&self) -> Result<Vec<dam_application::Conflict>, StoreError> {
        self.all_conflicts()
    }
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.drop_conflict(oid)
    }
    fn add_notice(&self, notice: &dam_application::Notice) -> Result<(), StoreError> {
        self.push_notice(notice)
    }
    fn notices(&self) -> Result<Vec<dam_application::Notice>, StoreError> {
        self.all_notices()
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        self.drop_notices()
    }
}

#[cfg(test)]
mod tests;
