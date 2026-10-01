use dam_application::{AcceptedKindsRepository, RemoteName, StoreError};
use dam_domain::Kind;
use rusqlite::{OptionalExtension, params};

use super::SqliteStore;
use super::objects::sql;

impl AcceptedKindsRepository for SqliteStore {
    fn accepted_kinds(&self, remote: &RemoteName) -> Result<Option<Vec<Kind>>, StoreError> {
        let words: Option<String> = self
            .conn
            .query_row(
                "SELECT kinds FROM accepted_kinds WHERE remote = ?1",
                params![remote.0],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        words
            .map(|words| {
                words
                    .split(',')
                    .filter(|word| !word.is_empty())
                    .map(|word| {
                        Kind::parse(word)
                            .ok_or_else(|| StoreError::Failed(format!("unknown kind {word:?}")))
                    })
                    .collect()
            })
            .transpose()
    }

    fn set_accepted_kinds(&self, remote: &RemoteName, kinds: &[Kind]) -> Result<(), StoreError> {
        let words: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
        self.conn
            .execute(
                "INSERT INTO accepted_kinds (remote, kinds) VALUES (?1, ?2) \
                 ON CONFLICT (remote) DO UPDATE SET kinds = excluded.kinds",
                params![remote.0, words.join(",")],
            )
            .map(|_| ())
            .map_err(sql)
    }
}
