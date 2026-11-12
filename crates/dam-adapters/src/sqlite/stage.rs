use dam_application::{StageRepository, StoreError};
use dam_domain::{Authored, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::authored_rows::{AUTHORED_COLUMNS, author_columns, change_columns, read_authored};
use super::objects::sql;

impl StageRepository for SqliteStore {
    fn put_staged(&self, staged: Authored) -> Result<(), StoreError> {
        let change = change_columns(&staged.change)?;
        let author = author_columns(&staged.author);
        self.conn
            .execute(
                "INSERT OR REPLACE INTO stage (oid, op, before_json, after_json, author_token, author_key_id, author_channel) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    staged.change.oid().as_str(),
                    change.op,
                    change.before,
                    change.after,
                    author.token,
                    author.key_id,
                    author.channel
                ],
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

    fn staged(&self) -> Result<Vec<Authored>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {AUTHORED_COLUMNS} FROM stage ORDER BY oid, author_token IS NULL"
            ))
            .map_err(sql)?;
        read_authored(&mut stmt, [])
    }
}

#[cfg(test)]
mod tests;
