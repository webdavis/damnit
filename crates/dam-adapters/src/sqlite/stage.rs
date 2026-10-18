use dam_application::{StageRepository, StoreError};
use dam_domain::{Change, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::codec::{object_to_json, op_to_text, read_changes};
use super::objects::sql;

impl StageRepository for SqliteStore {
    fn put_staged(&self, change: Change) -> Result<(), StoreError> {
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
}

#[cfg(test)]
mod tests;
