use dam_application::{Notice, NoticeRepository, StoreError};
use rusqlite::params;

use super::SqliteStore;
use super::codec::{notice_from_json, notice_to_json};
use super::objects::sql;

const NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM: i64 = 1000;

impl NoticeRepository for SqliteStore {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.in_savepoint("dam_notice", || {
            self.conn
                .execute(
                    "INSERT INTO notices (json) VALUES (?1)",
                    params![notice_to_json(notice)?],
                )
                .map_err(sql)?;
            self.conn
                .execute(
                    "DELETE FROM notices WHERE id NOT IN \
                     (SELECT id FROM notices ORDER BY id DESC LIMIT ?1)",
                    params![NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM],
                )
                .map(|_| ())
                .map_err(sql)
        })
    }

    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM notices ORDER BY id")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|j| notice_from_json(&j)))
            .collect()
    }

    fn remove_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM notices WHERE json = ?1",
                params![notice_to_json(notice)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn clear_notices(&self) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM notices", [])
            .map(|_| ())
            .map_err(sql)
    }
}

#[cfg(test)]
mod tests;
