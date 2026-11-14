use dam_application::{AnswerRepository, RemoteName, StoreError};
use dam_domain::Oid;
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::objects::sql;

impl AnswerRepository for SqliteStore {
    fn answers(&self, remote: &RemoteName, oid: &Oid) -> Result<u64, StoreError> {
        let count: Option<i64> = self
            .conn
            .query_row(
                "SELECT count FROM answers WHERE remote = ?1 AND oid = ?2",
                params![remote.0, oid.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        count
            .map(u64::try_from)
            .transpose()
            .map(Option::unwrap_or_default)
            .map_err(|e| StoreError::Failed(format!("a count of answers below zero: {e}")))
    }

    fn count_answer(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO answers (remote, oid, count) VALUES (?1, ?2, 1) \
                 ON CONFLICT (remote, oid) DO UPDATE SET count = count + 1",
                params![remote.0, oid.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}
