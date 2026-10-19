use dam_application::{ObjectRepository, StoreError};
use dam_domain::{Object, Oid};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::codec::{object_from_json, object_to_json};

pub(super) fn sql(e: rusqlite::Error) -> StoreError {
    match e.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
            StoreError::Busy(e.to_string())
        }
        _ => StoreError::Failed(e.to_string()),
    }
}

impl ObjectRepository for SqliteStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.conn
            .query_row(
                "SELECT json FROM objects WHERE oid = ?1",
                params![oid.as_str()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|j| object_from_json(&j))
            .transpose()
    }

    fn all(&self) -> Result<Vec<Object>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM objects ORDER BY path, oid")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|j| object_from_json(&j)))
            .collect()
    }

    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM objects WHERE json LIKE ?1")
            .map_err(sql)?;
        let rows = stmt
            .query_map(params![format!("%{}%", oid.as_str())], |r| {
                r.get::<_, String>(0)
            })
            .map_err(sql)?;
        let mut out = Vec::new();
        for row in rows {
            let object = object_from_json(&row.map_err(sql)?)?;
            if object.base().depends.contains(oid) {
                out.push(object.oid().clone());
            }
        }
        Ok(out)
    }

    fn put(&self, object: &Object) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO objects (oid, path, json) VALUES (?1, ?2, ?3)
                 ON CONFLICT(oid) DO UPDATE SET path = excluded.path, json = excluded.json",
                params![
                    object.oid().as_str(),
                    object.base().path.as_str(),
                    object_to_json(object)?
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn delete(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM objects WHERE oid = ?1", params![oid.as_str()])
            .map(|_| ())
            .map_err(sql)
    }

    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.conn
            .query_row(
                "SELECT json FROM committed WHERE oid = ?1",
                params![oid.as_str()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|j| object_from_json(&j))
            .transpose()
    }
}

#[cfg(test)]
mod tests;
