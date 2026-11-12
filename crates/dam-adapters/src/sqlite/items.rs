use dam_application::{ItemRepository, StoreError};
use dam_domain::{Item, ItemHash, Oid, Outcome, Timestamp};
use rusqlite::{OptionalExtension, Row, params};

use super::SqliteStore;
use super::authored_rows::{change_columns, token_columns, token_from_columns};
use super::codec::change_from_row;
use super::objects::sql;

const COLUMNS: &str =
    "hash, token, key_id, channel, op, before_json, after_json, created, outcome, outcome_at";

impl ItemRepository for SqliteStore {
    fn put_item(&self, item: &Item) -> Result<(), StoreError> {
        let change = change_columns(&item.change)?;
        let token = token_columns(&item.author);
        self.conn
            .execute(
                &format!("INSERT INTO items ({COLUMNS}, target) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"),
                params![
                    item.hash.as_str(),
                    token.token,
                    token.key_id,
                    token.channel,
                    change.op,
                    change.before,
                    change.after,
                    item.created.to_string(),
                    item.outcome.as_str(),
                    item.outcome.at().map(|at| at.to_string()),
                    item.target().as_str(),
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }

    fn item(&self, hash: &ItemHash) -> Result<Option<Item>, StoreError> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM items WHERE hash = ?1"),
                params![hash.as_str()],
                raw,
            )
            .optional()
            .map_err(sql)?
            .map(item_from)
            .transpose()
    }

    fn items(&self) -> Result<Vec<Item>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {COLUMNS} FROM items ORDER BY rowid"))
            .map_err(sql)?;
        let rows = stmt.query_map([], raw).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(item_from)).collect()
    }

    fn settle(&self, hash: &ItemHash, outcome: Outcome) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE items SET outcome = ?2, outcome_at = ?3 WHERE hash = ?1",
                params![
                    hash.as_str(),
                    outcome.as_str(),
                    outcome.at().map(|at| at.to_string())
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }
}

struct Raw {
    hash: String,
    token: String,
    key_id: String,
    channel: String,
    op: String,
    before: Option<String>,
    after: Option<String>,
    created: String,
    outcome: String,
    outcome_at: Option<String>,
}

fn raw(r: &Row<'_>) -> rusqlite::Result<Raw> {
    Ok(Raw {
        hash: r.get(0)?,
        token: r.get(1)?,
        key_id: r.get(2)?,
        channel: r.get(3)?,
        op: r.get(4)?,
        before: r.get(5)?,
        after: r.get(6)?,
        created: r.get(7)?,
        outcome: r.get(8)?,
        outcome_at: r.get(9)?,
    })
}

fn item_from(raw: Raw) -> Result<Item, StoreError> {
    let at = |text: &str| -> Result<Timestamp, StoreError> {
        text.parse()
            .map_err(|e: jiff::Error| StoreError::Failed(format!("stored item time: {e}")))
    };
    let outcome_at = raw.outcome_at.as_deref().map(at).transpose()?;
    Ok(Item {
        hash: ItemHash::new(Oid::parse(&raw.hash).map_err(|e| StoreError::Failed(e.to_string()))?),
        author: token_from_columns(raw.token, &raw.key_id, &raw.channel)?,
        change: change_from_row(&raw.op, raw.before, raw.after)?,
        created: at(&raw.created)?,
        outcome: Outcome::from_parts(&raw.outcome, outcome_at)
            .ok_or_else(|| StoreError::Failed(format!("stored item outcome {:?}", raw.outcome)))?,
    })
}
