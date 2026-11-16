use std::collections::BTreeMap;

use dam_domain::{Object, Oid, Path, nearest_parents};
use rusqlite::{Transaction, params};

use super::super::codec::object_to_json;

mod rows;

use rows::{Rows, every, failed, layer, rows};

type Filled = BTreeMap<Oid, (Path, Vec<Oid>)>;

pub(super) fn fill_every_parent(tx: &Transaction) -> Result<(), rusqlite::Error> {
    let mut seen = BTreeMap::new();
    for remote in remotes(tx)? {
        let copies = Rows {
            table: "snapshots",
            column: "json",
            remote: Some(&remote),
        };
        let objects = layer(tx, &copies)?;
        let filled = fill(tx, &copies, &[], &objects)?;
        seen.insert(remote, (objects, filled));
    }
    let every_seen: Vec<&Filled> = seen.values().map(|(_, filled)| filled).collect();
    let committed = layer(tx, &every("committed", "json"))?;
    let committed_filled = fill(tx, &every("committed", "json"), &every_seen, &committed)?;
    let working = layer(tx, &every("objects", "json"))?;
    let working_filled = fill(
        tx,
        &every("objects", "json"),
        &[&committed_filled],
        &working,
    )?;
    fill(
        tx,
        &every("follow_ons", "working_json"),
        &[&committed_filled],
        &working,
    )?;
    for table in ["stage", "items", "follow_ons"] {
        fill(
            tx,
            &every(table, "after_json"),
            &[&committed_filled],
            &working,
        )?;
        fill(
            tx,
            &every(table, "before_json"),
            &[&committed_filled],
            &committed,
        )?;
    }
    fill(
        tx,
        &every("conflicts", "ours_json"),
        &[&working_filled],
        &working,
    )?;
    for (remote, (copies, filled)) in &seen {
        let theirs = Rows {
            table: "conflicts",
            column: "theirs_json",
            remote: Some(remote),
        };
        fill(tx, &theirs, &[filled], copies)?;
    }
    Ok(())
}

fn remotes(tx: &Transaction) -> Result<Vec<String>, rusqlite::Error> {
    tx.prepare("SELECT DISTINCT remote FROM snapshots ORDER BY remote")?
        .query_map([], |r| r.get(0))?
        .collect()
}
fn fill(
    tx: &Transaction,
    of: &Rows,
    beneath: &[&Filled],
    nearest_among: &[Object],
) -> Result<Filled, rusqlite::Error> {
    let mut filled = Filled::new();
    for (rowid, mut object) in rows(tx, of)? {
        let path = object.base().path.clone();
        let parents = beneath
            .iter()
            .find_map(|below| below.get(object.oid()).filter(|(p, _)| *p == path))
            .map(|(_, parents)| parents.clone())
            .unwrap_or_else(|| nearest_parents(&path, nearest_among));
        object.base_mut().parent = match parents.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        };
        let json = object_to_json(&object).map_err(failed)?;
        let update = format!(
            "UPDATE {} SET {} = ?1 WHERE rowid = ?2",
            of.table, of.column
        );
        tx.execute(&update, params![json, rowid])?;
        filled.insert(object.oid().clone(), (path, parents));
    }
    Ok(filled)
}
