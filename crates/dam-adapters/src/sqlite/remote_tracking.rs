use dam_application::{RemoteName, RemoteTrackingRepository, StoreError};
use dam_domain::{Object, Oid, Timestamp};
use rusqlite::{Connection, OptionalExtension, params};

use super::SqliteStore;
use super::codec::{object_from_json, object_to_json};
use super::objects::sql;

#[derive(Clone, Copy)]
enum Stamp {
    Pull,
    Push,
}

impl Stamp {
    fn select(self) -> &'static str {
        match self {
            Stamp::Pull => "SELECT at FROM pulls WHERE remote = ?1",
            Stamp::Push => "SELECT at FROM pushes WHERE remote = ?1",
        }
    }

    fn upsert(self) -> &'static str {
        match self {
            Stamp::Pull => {
                "INSERT INTO pulls (remote, at) VALUES (?1, ?2) \
                 ON CONFLICT(remote) DO UPDATE SET at = excluded.at"
            }
            Stamp::Push => {
                "INSERT INTO pushes (remote, at) VALUES (?1, ?2) \
                 ON CONFLICT(remote) DO UPDATE SET at = excluded.at"
            }
        }
    }

    fn read(self, conn: &Connection, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        conn.query_row(self.select(), params![remote.0], |r| r.get::<_, String>(0))
            .optional()
            .map_err(sql)?
            .map(|t| {
                t.parse::<Timestamp>()
                    .map_err(|e| StoreError::Failed(e.to_string()))
            })
            .transpose()
    }

    fn write(
        self,
        conn: &Connection,
        remote: &RemoteName,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        conn.execute(self.upsert(), params![remote.0, at.to_string()])
            .map(|_| ())
            .map_err(sql)
    }
}

impl RemoteTrackingRepository for SqliteStore {
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        self.conn
            .query_row(
                "SELECT remote_id FROM remote_ids WHERE remote = ?1 AND oid = ?2",
                params![remote.0, oid.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)
    }

    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        self.conn
            .query_row(
                "SELECT oid FROM remote_ids WHERE remote = ?1 AND remote_id = ?2",
                params![remote.0, remote_id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|o| Oid::parse(&o).map_err(|e| StoreError::Failed(e.to_string())))
            .transpose()
    }

    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.in_savepoint("dam_remote_id", || {
            self.conn
                .execute(
                    "DELETE FROM remote_ids WHERE remote = ?1 AND (oid = ?2 OR remote_id = ?3)",
                    params![remote.0, oid.as_str(), remote_id],
                )
                .map_err(sql)?;
            self.conn
                .execute(
                    "INSERT INTO remote_ids (remote, oid, remote_id) VALUES (?1, ?2, ?3)",
                    params![remote.0, oid.as_str(), remote_id],
                )
                .map(|_| ())
                .map_err(sql)
        })
    }

    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        self.conn
            .query_row(
                "SELECT json FROM snapshots WHERE remote = ?1 AND oid = ?2",
                params![remote.0, oid.as_str()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|j| object_from_json(&j))
            .transpose()
    }

    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO snapshots (remote, oid, json) VALUES (?1, ?2, ?3)
                 ON CONFLICT(remote, oid) DO UPDATE SET json = excluded.json",
                params![remote.0, object.oid().as_str(), object_to_json(object)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.in_savepoint("dam_clear_mapping", || {
            for table in ["remote_ids", "snapshots"] {
                self.conn
                    .execute(
                        &format!("DELETE FROM {table} WHERE remote = ?1 AND oid = ?2"),
                        params![remote.0, oid.as_str()],
                    )
                    .map_err(sql)?;
            }
            Ok(())
        })
    }

    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT token FROM sync_tokens WHERE remote = ?1",
                params![remote.0],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(sql)?
            .flatten())
    }

    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO sync_tokens (remote, token) VALUES (?1, ?2)
                 ON CONFLICT(remote) DO UPDATE SET token = excluded.token",
                params![remote.0, token],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        Stamp::Pull.read(&self.conn, remote)
    }

    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        Stamp::Pull.write(&self.conn, remote, at)
    }

    fn last_push(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        Stamp::Push.read(&self.conn, remote)
    }

    fn set_last_push(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        Stamp::Push.write(&self.conn, remote, at)
    }
}

#[cfg(test)]
mod tests;
