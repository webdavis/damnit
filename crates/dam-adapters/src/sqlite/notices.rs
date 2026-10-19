use dam_application::{Notice, NoticeRepository, StoreError};
use rusqlite::params;

use super::SqliteStore;
use super::codec::{notice_from_json, notice_to_json};
use super::objects::sql;

const NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM: i64 = 1000;

impl NoticeRepository for SqliteStore {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.in_savepoint("dam_notice", || {
            self.conn
                .execute(
                    "INSERT INTO notices (json) VALUES (?1)",
                    params![notice_to_json(notice)?],
                )
                .map_err(sql)?;
            self.conn
                .execute(
                    "DELETE FROM notices WHERE id NOT IN \
                     (SELECT id FROM notices ORDER BY id DESC LIMIT ?1)",
                    params![NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM],
                )
                .map(|_| ())
                .map_err(sql)
        })
    }

    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM notices ORDER BY id")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|j| notice_from_json(&j)))
            .collect()
    }

    fn clear_notices(&self) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM notices", [])
            .map(|_| ())
            .map_err(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM;
    use dam_application::{Notice, NoticeRepository, RemoteName};

    #[test]
    fn a_notice_table_that_is_never_cleared_keeps_the_newest_and_drops_the_oldest() {
        let store = crate::sqlite::SqliteStore::in_memory().unwrap();
        for n in 0..NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM + 5 {
            store
                .add_notice(&Notice::PullFailed {
                    remote: RemoteName("t".into()),
                    why: format!("reason {n}"),
                })
                .unwrap();
        }
        let kept = store.notices().unwrap();
        assert_eq!(
            kept.len() as i64,
            NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM
        );
        let oldest = match &kept[0] {
            Notice::PullFailed { why, .. } => why.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(oldest, "reason 5", "the five oldest went");
    }
}
