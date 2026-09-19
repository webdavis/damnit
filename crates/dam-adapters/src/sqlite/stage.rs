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

    fn push_retries(&self, _remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn set_push_retries(&self, _remote: &RemoteName, _oids: &[Oid]) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn remote_id(&self, _remote: &RemoteName, _oid: &Oid) -> Result<Option<String>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn oid_for_remote_id(
        &self,
        _remote: &RemoteName,
        _remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn map_remote_id(
        &self,
        _remote: &RemoteName,
        _oid: &Oid,
        _remote_id: &str,
    ) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn remote_snapshot(
        &self,
        _remote: &RemoteName,
        _oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn set_remote_snapshot(
        &self,
        _remote: &RemoteName,
        _object: &Object,
    ) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn clear_remote_mapping(&self, _remote: &RemoteName, _oid: &Oid) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn sync_token(&self, _remote: &RemoteName) -> Result<Option<String>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn set_sync_token(&self, _remote: &RemoteName, _token: Option<&str>) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn mark_conflict(
        &self,
        _remote: &RemoteName,
        _oid: &Oid,
        _theirs: &Object,
    ) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn conflicts(&self) -> Result<Vec<dam_application::Conflict>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn clear_conflict(&self, _oid: &Oid) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn add_notice(&self, _notice: &dam_application::Notice) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn notices(&self) -> Result<Vec<dam_application::Notice>, StoreError> {
        Err(StoreError("not yet".into()))
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        Err(StoreError("not yet".into()))
    }
}

#[cfg(test)]
mod tests;
