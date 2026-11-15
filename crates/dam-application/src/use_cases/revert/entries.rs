use dam_domain::{Change, CommitRecord, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::use_cases::prefix::require_a_hex_prefix;

pub(super) fn named(record: &CommitRecord, texts: &[String]) -> Result<Vec<Change>, UseCaseError> {
    let changes = each_objects_whole_change(record);
    if texts.is_empty() {
        return Ok(changes);
    }
    let mut chosen = Vec::with_capacity(texts.len());
    for text in texts {
        chosen.push(the_one_in_the_commit(record, &changes, text)?);
    }
    Ok(changes
        .into_iter()
        .filter(|c| chosen.contains(c.oid()))
        .collect())
}

fn each_objects_whole_change(record: &CommitRecord) -> Vec<Change> {
    let mut oids: Vec<&Oid> = Vec::new();
    for entry in &record.changes {
        if !oids.contains(&entry.change.oid()) {
            oids.push(entry.change.oid());
        }
    }
    oids.into_iter()
        .filter_map(|oid| {
            let mut entries = record.changes.iter().map(|e| &e.change);
            let first = entries.clone().find(|c| c.oid() == oid)?;
            let last = entries.rfind(|c| c.oid() == oid)?;
            Change::between(first.before().cloned(), last.after().cloned())
        })
        .collect()
}

fn the_one_in_the_commit(
    record: &CommitRecord,
    changes: &[Change],
    text: &str,
) -> Result<Oid, UseCaseError> {
    require_a_hex_prefix(text)?;
    let matches: Vec<&Oid> = changes
        .iter()
        .map(Change::oid)
        .filter(|oid| oid.as_str().starts_with(text))
        .collect();
    match matches.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(Refusal::NotInCommit {
            commit: record.id.clone(),
            text: text.to_string(),
        }
        .into()),
        _ => Err(Refusal::AmbiguousPrefix {
            text: text.to_string(),
            matches: matches.iter().map(|oid| oid.to_string()).collect(),
        }
        .into()),
    }
}
