use dam_application::{StageRepository, StoreError};
use dam_domain::{Change, Oid, coalesce};
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

    fn coalesce_into_stage(&self, change: Change) -> Result<(), StoreError> {
        let merged = match self.staged_change(&change.oid)? {
            Some(existing) => coalesce(existing, change.clone()),
            None => Some(change.clone()),
        };
        match merged {
            Some(c) => self.write_stage(&c),
            None => self.unstage(&change.oid),
        }
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

impl StageRepository for SqliteStore {
    /// Reading the staged change, coalescing it and writing the result is one
    /// unit of work, so two dam processes staging the same object cannot
    /// interleave and lose one of the two coalesces.
    fn stage(&self, change: Change) -> Result<(), StoreError> {
        self.in_savepoint("dam_stage", || self.coalesce_into_stage(change))
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
