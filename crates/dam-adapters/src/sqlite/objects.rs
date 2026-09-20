use dam_application::{ObjectRepository, StoreError};
use dam_domain::{Object, Oid, Path};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::codec::{object_from_json, object_to_json};

/// The smallest string that sorts above every string starting with `prefix`,
/// under the byte-wise comparison SQLite uses for TEXT. A path always ends in
/// a separator, so raising that last byte is enough.
fn just_past(prefix: &str) -> String {
    let mut bytes = prefix.as_bytes().to_vec();
    match bytes.last_mut() {
        Some(last) if *last < u8::MAX => *last += 1,
        _ => bytes.push(0),
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A busy or locked store is its own outcome: the work did not happen and a
/// later attempt is the answer, which a flattened string cannot tell a caller.
pub(super) fn sql(e: rusqlite::Error) -> StoreError {
    match e.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
            StoreError::Busy(e.to_string())
        }
        _ => StoreError::Failed(e.to_string()),
    }
}

impl SqliteStore {
    pub(super) fn get_object(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
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

    pub(super) fn all_objects(&self) -> Result<Vec<Object>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM objects ORDER BY path, oid")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|j| object_from_json(&j)))
            .collect()
    }

    /// Direct children: path starts with the parent and has exactly one more segment.
    pub(super) fn children(&self, path: &Path) -> Result<Vec<Object>, StoreError> {
        let prefix = path.as_str();
        // A range rather than substr(path, ...), which is a function on the
        // column and cannot use the objects_path index. Every path is under
        // the root, so the root has a lower bound and no upper one.
        let upper = just_past(prefix);
        let (query, bounds): (&str, Vec<&dyn rusqlite::ToSql>) = if prefix.is_empty() {
            (
                "SELECT json, path FROM objects WHERE path > ?1",
                vec![&prefix],
            )
        } else {
            (
                "SELECT json, path FROM objects WHERE path > ?1 AND path < ?2",
                vec![&prefix, &upper],
            )
        };
        let mut stmt = self.conn.prepare(query).map_err(sql)?;
        let rows = stmt
            .query_map(bounds.as_slice(), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(sql)?;
        let mut out = Vec::new();
        for row in rows {
            let (json, child_path) = row.map_err(sql)?;
            let Some(tail) = child_path.strip_prefix(prefix) else {
                continue;
            };
            if tail.matches('/').count() == 1 {
                out.push(object_from_json(&json)?);
            }
        }
        Ok(out)
    }

    /// Every object that depends on `oid`. The stored JSON is filtered in SQL
    /// first, so only objects whose text holds the oid at all are decoded.
    pub(super) fn dependents(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
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

    pub(super) fn put_object(&self, object: &Object) -> Result<(), StoreError> {
        let kind = match object {
            Object::Task(_) => "task",
            Object::Event(_) => "event",
        };
        self.conn
            .execute(
                "INSERT INTO objects (oid, kind, path, json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(oid) DO UPDATE SET kind = excluded.kind, path = excluded.path, json = excluded.json",
                params![
                    object.oid().as_str(),
                    kind,
                    object.base().path.as_str(),
                    object_to_json(object)?
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn delete_object(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM objects WHERE oid = ?1", params![oid.as_str()])
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn committed_object(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
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

impl ObjectRepository for SqliteStore {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_domain::Object;

    fn failure(code: std::ffi::c_int) -> rusqlite::Error {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(code),
            Some("database is locked".into()),
        )
    }

    #[test]
    fn children_of_the_root_are_found_and_deeper_objects_are_not() {
        let store = crate::sqlite::SqliteStore::in_memory().unwrap();
        let put = |path: &str, byte: u8| {
            let mut t =
                dam_domain::Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(byte)), "milk");
            t.base.path = Path::parse(path).unwrap();
            store.put_object(&Object::Task(t)).unwrap();
        };
        put("", 1);
        put("work", 2);
        put("work/deep", 3);
        let root: Vec<String> = store
            .children(&Path::parse("").unwrap())
            .unwrap()
            .iter()
            .map(|o| o.base().path.as_str().to_string())
            .collect();
        assert_eq!(root, vec!["work/".to_string()]);
        let under_work: Vec<String> = store
            .children(&Path::parse("work").unwrap())
            .unwrap()
            .iter()
            .map(|o| o.base().path.as_str().to_string())
            .collect();
        assert_eq!(under_work, vec!["work/deep/".to_string()]);
    }

    #[test]
    fn the_upper_bound_sits_above_every_path_under_the_prefix() {
        assert_eq!(just_past("work/"), "work0");
        assert!("work/a" < just_past("work/").as_str());
        assert!("work/z/deep/" < just_past("work/").as_str());
        assert!("work0" >= just_past("work/").as_str());
    }

    #[test]
    fn a_busy_or_locked_store_is_its_own_outcome() {
        assert!(matches!(
            sql(failure(rusqlite::ffi::SQLITE_BUSY)),
            StoreError::Busy(_)
        ));
        assert!(matches!(
            sql(failure(rusqlite::ffi::SQLITE_LOCKED)),
            StoreError::Busy(_)
        ));
    }

    #[test]
    fn every_other_store_failure_stays_a_plain_failure() {
        assert!(matches!(
            sql(failure(rusqlite::ffi::SQLITE_CORRUPT)),
            StoreError::Failed(_)
        ));
        assert!(matches!(
            sql(failure(rusqlite::ffi::SQLITE_FULL)),
            StoreError::Failed(_)
        ));
        assert!(matches!(
            sql(rusqlite::Error::QueryReturnedNoRows),
            StoreError::Failed(_)
        ));
    }
}
