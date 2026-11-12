use std::collections::BTreeMap;

use dam_application::{OriginRepository, StoreError};
use dam_domain::{Oid, Origin};

use super::SqliteStore;
use super::authored_rows::author_from_columns;
use super::details_json::details_from_json;
use super::objects::sql;

type OriginRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl OriginRepository for SqliteStore {
    fn origins(&self) -> Result<BTreeMap<Oid, Origin>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT cc.oid, cc.author_token, cc.author_key_id, cc.author_channel, cc.details_json
                 FROM commit_changes AS cc JOIN commits AS c ON c.id = cc.commit_id
                 WHERE cc.op = 'create' ORDER BY c.seq, cc.ord",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(sql)?;
        let mut origins = BTreeMap::new();
        for row in rows {
            let (oid, token, key_id, channel, details): OriginRow = row.map_err(sql)?;
            let oid = Oid::parse(&oid).map_err(|e| StoreError::Failed(e.to_string()))?;
            if let std::collections::btree_map::Entry::Vacant(slot) = origins.entry(oid) {
                slot.insert(Origin {
                    author: author_from_columns(token, key_id, channel)?,
                    details: details_from_json(details)?,
                });
            }
        }
        Ok(origins)
    }
}
