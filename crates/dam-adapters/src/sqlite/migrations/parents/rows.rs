use dam_domain::Object;
use rusqlite::Transaction;

use super::super::super::codec::object_from_json;

pub(super) struct Rows<'a> {
    pub(super) table: &'a str,
    pub(super) column: &'a str,
    pub(super) remote: Option<&'a str>,
}

pub(super) const fn every<'a>(table: &'a str, column: &'a str) -> Rows<'a> {
    Rows {
        table,
        column,
        remote: None,
    }
}

pub(super) fn rows(tx: &Transaction, rows: &Rows) -> Result<Vec<(i64, Object)>, rusqlite::Error> {
    let Rows {
        table,
        column,
        remote,
    } = rows;
    let filter = if remote.is_some() {
        "remote = ?1"
    } else {
        "?1 IS NULL"
    };
    let key = key_of(table);
    let query = format!(
        "SELECT rowid, {key}, {column} FROM {table} WHERE {column} IS NOT NULL AND {filter}"
    );
    let stored: Vec<(i64, String, String)> = tx
        .prepare(&query)?
        .query_map([remote], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    stored
        .into_iter()
        .map(|(rowid, id, json)| {
            let object = object_from_json(&json)
                .map_err(|e| unreadable(format!("{table} {key} {id}: {}", e.why())))?;
            Ok((rowid, object))
        })
        .collect()
}

fn key_of(table: &str) -> &'static str {
    match table {
        "items" => "target",
        "follow_ons" => "hash",
        _ => "oid",
    }
}

pub(super) fn layer(tx: &Transaction, of: &Rows) -> Result<Vec<Object>, rusqlite::Error> {
    Ok(rows(tx, of)?.into_iter().map(|(_, o)| o).collect())
}

pub(super) fn failed(e: dam_application::StoreError) -> rusqlite::Error {
    unreadable(e.why().to_string())
}

fn unreadable(why: String) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(why.into())
}
