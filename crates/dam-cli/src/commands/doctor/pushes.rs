use dam_adapters::SqliteStore;

use super::check::{Check, Outcome};
use super::helpers::{CONFIG_UNREAD, NO_REMOTES};
use super::machine::Machine;
use super::row::Row;
use super::store_reading::{Owed, Store};
use crate::args::CheckName;

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let Ok(config) = machine.config() else {
        return vec![found(Outcome::NotRun, CONFIG_UNREAD.into())];
    };
    if config.remotes.is_empty() {
        return vec![found(Outcome::NotRun, NO_REMOTES.into())];
    }
    let owed = match machine.store() {
        Ok(Store::NoneYet) => None,
        Ok(Store::Read(read)) if read.version == SqliteStore::SCHEMA_VERSION => Some(&read.owed),
        Ok(Store::Read(_)) => {
            return vec![not_checked("the database is not at this dam's version")];
        }
        Ok(Store::LeftOver(_)) => return vec![not_checked("the database was not read")],
        Err(_) => return vec![not_checked("the database did not open")],
    };
    let rows = config.remotes.iter().enumerate().map(|(index, remote)| {
        let name = &remote.name.0;
        match owed.and_then(|owed| owed.get(index)) {
            None
            | Some(Ok(Owed {
                changes: 0,
                unanswered: 0,
            })) => Row::of(name, Outcome::Ok, "nothing owed"),
            Some(Ok(owed)) => said(name, owed),
            Some(Err(error)) => {
                let detail = format!("not checked, the database did not read: {error}");
                Row::of(name, Outcome::NotRun, detail)
            }
        }
    });
    vec![Check::of_rows(CheckName::Pushes, rows.collect())]
}

fn found(result: Outcome, detail: String) -> Check {
    Check::of(CheckName::Pushes, result, detail)
}

fn not_checked(why: &str) -> Check {
    found(Outcome::NotRun, format!("not checked, {why}"))
}

fn said(remote: &str, owed: &Owed) -> Row {
    let changes = |n: usize| match n {
        1 => "1 change".to_string(),
        n => format!("{n} changes"),
    };
    let unanswered = owed.unanswered;
    let owed = (owed.changes > 0).then(|| format!("{} still owed", changes(owed.changes)));
    let unanswered =
        (unanswered > 0).then(|| format!("{} sent without an answer", changes(unanswered)));
    let detail: Vec<String> = owed.into_iter().chain(unanswered).collect();
    Row::of(remote, Outcome::Warning, detail.join(", ")).advising("run dam push")
}
