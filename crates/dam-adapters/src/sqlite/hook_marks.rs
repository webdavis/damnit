use dam_application::{HookEvent, HookMarkRepository, StoreError};
use dam_domain::Oid;
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::objects::sql;

impl HookMarkRepository for SqliteStore {
    fn mark_hooked(&self, oid: &Oid, event: HookEvent) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO hook_marks (oid, event) VALUES (?1, ?2) \
                 ON CONFLICT (oid) DO UPDATE SET event = excluded.event",
                params![oid.as_str(), event.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn hooked(&self, oid: &Oid) -> Result<Option<HookEvent>, StoreError> {
        let word: Option<String> = self
            .conn
            .query_row(
                "SELECT event FROM hook_marks WHERE oid = ?1",
                params![oid.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        word.map(|word| {
            HookEvent::ALL
                .into_iter()
                .find(|event| event.as_str() == word)
                .ok_or_else(|| StoreError::Failed(format!("unknown hook {word:?}")))
        })
        .transpose()
    }

    fn forget_hooked(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM hook_marks WHERE oid = ?1",
                params![oid.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}
