use dam_application::{RemoteName, RemoteTrackingRepository, StoreError};
use dam_domain::{Object, Oid, Timestamp};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::codec::{object_from_json, object_to_json};
use super::objects::sql;

impl SqliteStore {
    pub(super) fn remote_id_of(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<String>, StoreError> {
        self.conn
            .query_row(
                "SELECT remote_id FROM remote_ids WHERE remote = ?1 AND oid = ?2",
                params![remote.0, oid.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)
    }

    pub(super) fn oid_of_remote_id(
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

    pub(super) fn set_remote_id(
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
                .map_err(sql)?;
            Ok(())
        })
    }

    pub(super) fn snapshot_of(
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

    pub(super) fn set_snapshot(
        &self,
        remote: &RemoteName,
        object: &Object,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO snapshots (remote, oid, json) VALUES (?1, ?2, ?3)
                 ON CONFLICT(remote, oid) DO UPDATE SET json = excluded.json",
                params![remote.0, object.oid().as_str(), object_to_json(object)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn clear_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.in_savepoint("dam_clear_mapping", || {
            self.conn
                .execute(
                    "DELETE FROM remote_ids WHERE remote = ?1 AND oid = ?2",
                    params![remote.0, oid.as_str()],
                )
                .map_err(sql)?;
            self.conn
                .execute(
                    "DELETE FROM snapshots WHERE remote = ?1 AND oid = ?2",
                    params![remote.0, oid.as_str()],
                )
                .map_err(sql)?;
            Ok(())
        })
    }

    pub(super) fn token_of(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
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

    pub(super) fn set_token(
        &self,
        remote: &RemoteName,
        token: Option<&str>,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO sync_tokens (remote, token) VALUES (?1, ?2)
                 ON CONFLICT(remote) DO UPDATE SET token = excluded.token",
                params![remote.0, token],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn last_pull_of(
        &self,
        remote: &RemoteName,
    ) -> Result<Option<Timestamp>, StoreError> {
        self.conn
            .query_row(
                "SELECT at FROM pulls WHERE remote = ?1",
                params![remote.0],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|t| {
                t.parse::<Timestamp>()
                    .map_err(|e| StoreError::Failed(e.to_string()))
            })
            .transpose()
    }

    pub(super) fn set_last_pull_of(
        &self,
        remote: &RemoteName,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO pulls (remote, at) VALUES (?1, ?2)
                 ON CONFLICT(remote) DO UPDATE SET at = excluded.at",
                params![remote.0, at.to_string()],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn last_push_of(
        &self,
        remote: &RemoteName,
    ) -> Result<Option<Timestamp>, StoreError> {
        self.conn
            .query_row(
                "SELECT at FROM pushes WHERE remote = ?1",
                params![remote.0],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|t| {
                t.parse::<Timestamp>()
                    .map_err(|e| StoreError::Failed(e.to_string()))
            })
            .transpose()
    }

    pub(super) fn set_last_push_of(
        &self,
        remote: &RemoteName,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO pushes (remote, at) VALUES (?1, ?2)
                 ON CONFLICT(remote) DO UPDATE SET at = excluded.at",
                params![remote.0, at.to_string()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

impl RemoteTrackingRepository for SqliteStore {
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        self.remote_id_of(remote, oid)
    }
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        self.oid_of_remote_id(remote, remote_id)
    }
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.set_remote_id(remote, oid, remote_id)
    }
    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        self.snapshot_of(remote, oid)
    }
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.set_snapshot(remote, object)
    }
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.clear_mapping(remote, oid)
    }
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        self.token_of(remote)
    }
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        self.set_token(remote, token)
    }
    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        self.last_pull_of(remote)
    }
    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        self.set_last_pull_of(remote, at)
    }
    fn last_push(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        self.last_push_of(remote)
    }
    fn set_last_push(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        self.set_last_push_of(remote, at)
    }
}

#[cfg(test)]
mod tests;
