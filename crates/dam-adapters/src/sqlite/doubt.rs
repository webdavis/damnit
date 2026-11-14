use std::collections::BTreeSet;

use dam_application::{DoubtRepository, FieldsInDoubt, RemoteName, StoreError};
use dam_domain::{Field, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::objects::sql;

impl DoubtRepository for SqliteStore {
    fn fields_in_doubt(&self, remote: &RemoteName) -> Result<FieldsInDoubt, StoreError> {
        let mut rows = self
            .conn
            .prepare("SELECT oid, field FROM fields_in_doubt WHERE remote = ?1")
            .map_err(sql)?;
        let rows = rows
            .query_map(params![remote.0], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(sql)?;
        let mut doubt = FieldsInDoubt::new();
        for row in rows {
            let (oid, field) = row.map_err(sql)?;
            let oid = Oid::parse(&oid).map_err(|e| StoreError::Failed(e.to_string()))?;
            let field = Field::parse(&field)
                .ok_or_else(|| StoreError::Failed(format!("unknown field {field:?}")))?;
            doubt.entry(oid).or_default().insert(field);
        }
        Ok(doubt)
    }

    fn set_fields_in_doubt(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        fields: &BTreeSet<Field>,
    ) -> Result<(), StoreError> {
        self.in_savepoint("dam_fields_in_doubt", || {
            self.conn
                .execute(
                    "DELETE FROM fields_in_doubt WHERE remote = ?1 AND oid = ?2",
                    params![remote.0, oid.as_str()],
                )
                .map_err(sql)?;
            for field in fields {
                self.conn
                    .execute(
                        "INSERT INTO fields_in_doubt (remote, oid, field) VALUES (?1, ?2, ?3)",
                        params![remote.0, oid.as_str(), field.as_str()],
                    )
                    .map_err(sql)?;
            }
            Ok(())
        })
    }
}
