use dam_application::{RemoteName, StoreError};
use dam_domain::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use crate::sqlite::objects::sql;

#[derive(Clone, Copy)]
pub(super) enum Stamp {
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

    pub(super) fn read(
        self,
        conn: &Connection,
        remote: &RemoteName,
    ) -> Result<Option<Timestamp>, StoreError> {
        conn.query_row(self.select(), params![remote.0], |r| r.get::<_, String>(0))
            .optional()
            .map_err(sql)?
            .map(|t| {
                t.parse::<Timestamp>()
                    .map_err(|e| StoreError::Failed(e.to_string()))
            })
            .transpose()
    }

    pub(super) fn write(
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
