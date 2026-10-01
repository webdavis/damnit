use dam_application::{StoreError, Transactional, UseCaseError};

use super::SqliteStore;
use super::objects;

impl SqliteStore {
    pub(super) fn in_savepoint<T, E: From<StoreError>>(
        &self,
        name: &'static str,
        work: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let (open, commit, abort) = if self.conn.is_autocommit() {
            outermost_unit_taking_the_write_lock_at_its_first_statement()
        } else {
            savepoint_nested_in_the_open_unit(name)
        };
        self.conn
            .execute_batch(&open)
            .map_err(|e| E::from(objects::sql(e)))?;
        match work() {
            Ok(value) => {
                self.conn
                    .execute_batch(&commit)
                    .map_err(|e| E::from(objects::sql(e)))?;
                Ok(value)
            }
            Err(e) => {
                let _ = self.conn.execute_batch(&abort);
                Err(e)
            }
        }
    }
}

fn outermost_unit_taking_the_write_lock_at_its_first_statement() -> (String, String, String) {
    (
        "BEGIN IMMEDIATE".to_string(),
        "COMMIT".to_string(),
        "ROLLBACK".to_string(),
    )
}

fn savepoint_nested_in_the_open_unit(name: &str) -> (String, String, String) {
    (
        format!("SAVEPOINT {name}"),
        format!("RELEASE {name}"),
        format!("ROLLBACK TO {name}; RELEASE {name}"),
    )
}

impl Transactional for SqliteStore {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        self.in_savepoint("dam_unit_of_work", work)
    }
}
