use dam_application::{
    Attempt, AttemptRepository, Attempts, Declared, MutationOp, RemoteName, StoreError,
};
use dam_domain::{Field, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::objects::sql;

impl AttemptRepository for SqliteStore {
    fn attempts(&self, remote: &RemoteName) -> Result<Attempts, StoreError> {
        let mut rows = self
            .conn
            .prepare(
                "SELECT oid, key, op, container_remote_id, declared FROM attempts \
                 WHERE remote = ?1 ORDER BY oid, position",
            )
            .map_err(sql)?;
        let rows = rows
            .query_map(params![remote.0], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(sql)?;
        let mut attempts = Attempts::new();
        for row in rows {
            let (oid, key, op, container_remote_id, declared) = row.map_err(sql)?;
            let oid = Oid::parse(&oid).map_err(|e| unreadable(e.to_string()))?;
            attempts.entry(oid).or_default().push(Attempt {
                key,
                op: op_from(&op)?,
                container_remote_id,
                declared: declared_from(&declared)?,
            });
        }
        Ok(attempts)
    }

    fn set_attempts(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        attempts: &[Attempt],
    ) -> Result<(), StoreError> {
        self.in_savepoint("dam_attempts", || {
            self.conn
                .execute(
                    "DELETE FROM attempts WHERE remote = ?1 AND oid = ?2",
                    params![remote.0, oid.as_str()],
                )
                .map_err(sql)?;
            for (position, attempt) in (0_i64..).zip(attempts) {
                self.conn
                    .execute(
                        "INSERT INTO attempts \
                         (remote, oid, position, key, op, container_remote_id, declared) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            remote.0,
                            oid.as_str(),
                            position,
                            attempt.key,
                            op_name(attempt.op),
                            attempt.container_remote_id,
                            declared_to(&attempt.declared)?,
                        ],
                    )
                    .map_err(sql)?;
            }
            Ok(())
        })
    }
}

fn op_name(op: MutationOp) -> &'static str {
    match op {
        MutationOp::Create => "create",
        MutationOp::Update => "update",
        MutationOp::Delete => "delete",
    }
}

fn op_from(name: &str) -> Result<MutationOp, StoreError> {
    [MutationOp::Create, MutationOp::Update, MutationOp::Delete]
        .into_iter()
        .find(|op| op_name(*op) == name)
        .ok_or_else(|| unreadable(format!("unknown operation {name:?}")))
}

fn declared_to(declared: &Declared) -> Result<String, StoreError> {
    let pairs: Vec<(&str, &str)> = declared
        .iter()
        .map(|(field, value)| (field.as_str(), value.as_str()))
        .collect();
    serde_json::to_string(&pairs).map_err(|e| unreadable(e.to_string()))
}

fn declared_from(json: &str) -> Result<Declared, StoreError> {
    let pairs: Vec<(String, String)> =
        serde_json::from_str(json).map_err(|e| unreadable(e.to_string()))?;
    pairs
        .into_iter()
        .map(|(name, value)| {
            let field =
                Field::parse(&name).ok_or_else(|| unreadable(format!("unknown field {name:?}")))?;
            Ok((field, value))
        })
        .collect()
}

fn unreadable(why: String) -> StoreError {
    StoreError::Failed(format!("a stored attempt: {why}"))
}
