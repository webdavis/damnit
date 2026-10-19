use dam_application::{Conflict, ConflictRepository, RemoteName, StoreError};
use dam_domain::Oid;
use rusqlite::params;

use super::SqliteStore;
use super::codec::{object_from_json, object_to_json};
use super::objects::sql;

impl ConflictRepository for SqliteStore {
    fn mark_conflict(&self, conflict: &Conflict) -> Result<(), StoreError> {
        let Conflict {
            remote,
            oid,
            ours,
            theirs,
        } = conflict;
        self.conn
            .execute(
                "INSERT INTO conflicts (oid, remote, ours_json, theirs_json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(oid) DO UPDATE SET remote = excluded.remote, ours_json = excluded.ours_json, theirs_json = excluded.theirs_json",
                params![oid.as_str(), remote.0, object_to_json(ours)?, object_to_json(theirs)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT oid, remote, ours_json, theirs_json FROM conflicts ORDER BY oid")
            .map_err(sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(sql)?;
        rows.map(|r| {
            let (oid, remote, ours, theirs) = r.map_err(sql)?;
            Ok(Conflict {
                remote: RemoteName(remote),
                oid: Oid::parse(&oid).map_err(|e| StoreError::Failed(e.to_string()))?,
                ours: object_from_json(&ours)?,
                theirs: object_from_json(&theirs)?,
            })
        })
        .collect()
    }

    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM conflicts WHERE oid = ?1",
                params![oid.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}
