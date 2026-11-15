use dam_application::{FollowOn, FollowOnRepository, ObjectState, StoreError};
use dam_domain::{Authored, ItemHash};
use rusqlite::{Row, params};

use super::SqliteStore;
use super::authored_rows::{author_columns, author_from_columns, change_columns};
use super::codec::{change_from_row, object_from_json, object_to_json};
use super::details_json::{details_from_json, details_to_json};
use super::objects::sql;

const STATE_COLUMNS: &str = "working_json, op, before_json, after_json, author_token, author_key_id, author_channel, details_json";

impl FollowOnRepository for SqliteStore {
    fn keep_follow_ons(&self, item: &ItemHash, follow_ons: &[FollowOn]) -> Result<(), StoreError> {
        self.forget_follow_ons(item)?;
        for (position, follow_on) in (0_i64..).zip(follow_ons) {
            self.put_state(item, position, "before", &follow_on.before)?;
            self.put_state(item, position, "after", &follow_on.after)?;
        }
        Ok(())
    }

    fn follow_ons(&self, item: &ItemHash) -> Result<Vec<FollowOn>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {STATE_COLUMNS} FROM follow_ons WHERE hash = ?1 ORDER BY position, moment = 'after'"
            ))
            .map_err(sql)?;
        let states = stmt
            .query_map(params![item.as_str()], raw)
            .map_err(sql)?
            .map(|r| r.map_err(sql).and_then(state_from))
            .collect::<Result<Vec<ObjectState>, StoreError>>()?;
        states
            .chunks(2)
            .map(|pair| match pair {
                [before, after] => Ok(FollowOn {
                    before: before.clone(),
                    after: after.clone(),
                }),
                _ => Err(StoreError::Failed(
                    "a stored follow-on is half written".into(),
                )),
            })
            .collect()
    }

    fn forget_follow_ons(&self, item: &ItemHash) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM follow_ons WHERE hash = ?1",
                params![item.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

impl SqliteStore {
    fn put_state(
        &self,
        item: &ItemHash,
        position: i64,
        moment: &str,
        state: &ObjectState,
    ) -> Result<(), StoreError> {
        let staged = state.owners_staged.as_ref();
        let change = staged.map(|s| change_columns(&s.change)).transpose()?;
        let author = staged.map(|s| author_columns(&s.author));
        self.conn
            .execute(
                &format!("INSERT INTO follow_ons (hash, position, moment, {STATE_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"),
                params![
                    item.as_str(),
                    position,
                    moment,
                    object_to_json(&state.working)?,
                    change.as_ref().map(|c| c.op),
                    change.as_ref().and_then(|c| c.before.clone()),
                    change.as_ref().and_then(|c| c.after.clone()),
                    author.as_ref().and_then(|a| a.token),
                    author.as_ref().and_then(|a| a.key_id),
                    author.as_ref().and_then(|a| a.channel),
                    details_to_json(staged.and_then(|s| s.details.as_ref()))?,
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

type Raw = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn raw(r: &Row<'_>) -> rusqlite::Result<Raw> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
    ))
}

fn state_from(raw: Raw) -> Result<ObjectState, StoreError> {
    let (working, op, before, after, token, key_id, channel, details) = raw;
    let owners_staged = op
        .map(|op| -> Result<Authored, StoreError> {
            Ok(Authored {
                change: change_from_row(&op, before, after)?,
                author: author_from_columns(token, key_id, channel)?,
                details: details_from_json(details)?,
            })
        })
        .transpose()?;
    Ok(ObjectState {
        working: object_from_json(&working)?,
        owners_staged,
    })
}
