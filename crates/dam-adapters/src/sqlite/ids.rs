use dam_application::{IdRepository, StoreError};
use dam_domain::{CallerId, IdHolder, IdKey, Oid};
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::objects::sql;

fn holder_columns(holder: &IdHolder) -> (&'static str, &str) {
    match holder {
        IdHolder::Owner => ("owner", ""),
        IdHolder::Token(name) => ("token", name.as_str()),
    }
}

fn columns(key: &IdKey) -> (&'static str, &str, &str, &str) {
    let (holder, token) = holder_columns(&key.holder);
    (
        holder,
        token,
        key.session.as_deref().unwrap_or(""),
        key.id.as_str(),
    )
}

impl IdRepository for SqliteStore {
    fn oid_for_id(&self, key: &IdKey) -> Result<Option<Oid>, StoreError> {
        let (holder, token, session, id) = columns(key);
        self.conn
            .query_row(
                "SELECT oid FROM caller_ids WHERE holder = ?1 AND token = ?2 AND session = ?3 AND id = ?4",
                params![holder, token, session, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|text| Oid::parse(&text).map_err(|e| StoreError::Failed(e.to_string())))
            .transpose()
    }

    fn id_for_oid(&self, holder: &IdHolder, oid: &Oid) -> Result<Option<CallerId>, StoreError> {
        let (holder, token) = holder_columns(holder);
        self.conn
            .query_row(
                "SELECT id FROM caller_ids WHERE holder = ?1 AND token = ?2 AND oid = ?3 LIMIT 1",
                params![holder, token, oid.as_str()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(sql)?
            .map(|text| CallerId::parse(&text).map_err(|e| StoreError::Failed(e.to_string())))
            .transpose()
    }

    fn map_id(&self, key: &IdKey, oid: &Oid) -> Result<(), StoreError> {
        let (holder, token, session, id) = columns(key);
        self.conn
            .execute(
                "INSERT INTO caller_ids (holder, token, session, id, oid) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (holder, token, session, id) DO UPDATE SET oid = excluded.oid",
                params![holder, token, session, id, oid.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}
