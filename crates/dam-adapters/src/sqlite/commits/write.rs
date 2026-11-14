use dam_application::StoreError;
use dam_domain::{Authored, CommitRecord};
use rusqlite::{Connection, params};

use crate::sqlite::authored_rows::{author_columns, change_columns};
use crate::sqlite::details_json::details_to_json;
use crate::sqlite::objects::sql;

pub(super) fn write_commit(conn: &Connection, record: &CommitRecord) -> Result<(), StoreError> {
    let seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM commits", [], |r| {
            r.get(0)
        })
        .map_err(sql)?;
    conn.execute(
        "INSERT INTO commits (id, seq, message, at) VALUES (?1, ?2, ?3, ?4)",
        params![
            record.id.as_str(),
            seq,
            record.message,
            record.at.to_string()
        ],
    )
    .map_err(sql)?;
    for (ord, change) in record.changes.iter().enumerate() {
        write_change(conn, record, ord, change)?;
    }
    Ok(())
}

fn write_change(
    conn: &Connection,
    record: &CommitRecord,
    ord: usize,
    authored: &Authored,
) -> Result<(), StoreError> {
    let oid = authored.change.oid().as_str();
    let change = change_columns(&authored.change)?;
    let author = author_columns(&authored.author);
    conn.execute(
        "INSERT INTO commit_changes (commit_id, ord, oid, op, before_json, after_json, author_token, author_key_id, author_channel, details_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            record.id.as_str(),
            ord as i64,
            oid,
            change.op,
            change.before,
            change.after,
            author.token,
            author.key_id,
            author.channel,
            details_to_json(authored.details.as_ref())?
        ],
    )
    .map_err(sql)?;
    match change.after {
        Some(after) => conn.execute(
            "INSERT INTO committed (oid, json) VALUES (?1, ?2)
             ON CONFLICT(oid) DO UPDATE SET json = excluded.json",
            params![oid, after],
        ),
        None => conn.execute("DELETE FROM committed WHERE oid = ?1", params![oid]),
    }
    .map_err(sql)?;
    conn.execute("DELETE FROM stage WHERE oid = ?1", params![oid])
        .map(|_| ())
        .map_err(sql)
}
