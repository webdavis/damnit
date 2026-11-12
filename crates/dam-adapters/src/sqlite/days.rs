use dam_application::{DayRepository, StoreError};
use dam_domain::{Date, Day};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::objects::sql;
use crate::day_doc::{day_from_doc, day_to_doc};

fn stored(json: &str) -> Result<Day, StoreError> {
    serde_json::from_str(json)
        .map_err(|e| e.to_string())
        .and_then(day_from_doc)
        .map_err(|e| StoreError::Failed(format!("stored day: {e}")))
}

impl DayRepository for SqliteStore {
    fn day(&self, on: Date) -> Result<Option<Day>, StoreError> {
        self.conn
            .query_row(
                "SELECT json FROM days WHERE day = ?1",
                params![on.to_string()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|json| stored(&json))
            .transpose()
    }

    fn put_day(&self, day: &Day) -> Result<(), StoreError> {
        let json = serde_json::to_string(&day_to_doc(day))
            .map_err(|e| StoreError::Failed(format!("stored day: {e}")))?;
        self.conn
            .execute(
                "INSERT INTO days (day, json) VALUES (?1, ?2) \
                 ON CONFLICT (day) DO UPDATE SET json = excluded.json",
                params![day.day.to_string(), json],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn days(&self) -> Result<Vec<Day>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM days ORDER BY day")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|json| stored(&json)))
            .collect()
    }
}
