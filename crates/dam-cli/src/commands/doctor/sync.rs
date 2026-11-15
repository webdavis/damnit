use dam_application::{ConfiguredDuration, RemoteConfig};

use super::helpers::CONFIG_UNREAD;
use super::machine::Machine;
use super::store_reading::{Store, Synced};
use crate::args::CheckName;
use crate::commands::check_report::{Check, LastRun, Outcome, Row};
use crate::layout::ago_to_one_unit_and_never_in_the_future;

const NOT_READ: &str = "not checked, the database was not read";

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let found = |result, detail: &str| Check::of(CheckName::Sync, result, detail);
    let check = match machine.config() {
        Err(_) => found(Outcome::NotRun, CONFIG_UNREAD),
        Ok(config) => {
            let rows: Vec<Row> = (config.remotes.iter().enumerate())
                .filter_map(|(index, remote)| {
                    Some(row(machine, index, remote, remote.sync_every.as_ref()?))
                })
                .collect();
            match rows.is_empty() {
                true => found(Outcome::NotRun, "no remote has sync_every"),
                false => Check::of_rows(CheckName::Sync, rows),
            }
        }
    };
    let scheduling = &machine.scheduling;
    let installed = scheduling.schedule.installed();
    vec![check.naming(LastRun::of(&scheduling.last_run, scheduling.now, installed))]
}

fn row(machine: &Machine, index: usize, remote: &RemoteConfig, every: &ConfiguredDuration) -> Row {
    let name = &remote.name.0;
    let found = |result, detail: String| Row::of(name, result, detail);
    if !machine.scheduling.schedule.installed() {
        let detail = "sync_every is set and the schedule is not installed";
        return found(Outcome::Warning, detail.into()).advising("run dam sync enable");
    }
    let synced = match synced(machine, index) {
        Err(why) => return found(Outcome::NotRun, why),
        Ok(synced) => synced.unwrap_or(&Synced::NEVER),
    };
    if let Some(why) = &synced.failed {
        let detail = format!("last sync failed: {why}");
        return found(Outcome::Warning, detail).advising(format!("run dam pull {name}"));
    }
    let when = match synced.last_pull {
        None => "not synced yet".to_string(),
        Some(at) => {
            let now = machine.scheduling.now;
            format!(
                "last synced {}",
                ago_to_one_unit_and_never_in_the_future(now, at)
            )
        }
    };
    found(Outcome::Ok, format!("every {}, {when}", every.text))
}

fn synced<'m>(machine: &'m Machine, index: usize) -> Result<Option<&'m Synced>, String> {
    match machine.store() {
        Ok(Store::NoneYet) => Ok(None),
        Ok(Store::Read(read)) => match read.synced.get(index) {
            Some(Ok(synced)) => Ok(Some(synced)),
            Some(Err(error)) => Err(format!("not checked, the database did not read: {error}")),
            None => Err(NOT_READ.into()),
        },
        Ok(Store::LeftOver(_)) | Err(_) => Err(NOT_READ.into()),
    }
}
