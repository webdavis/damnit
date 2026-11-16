use dam_adapters::SqliteStore;

use super::machine::Machine;
use super::store_reading::{Store, Unsettled};
use crate::args::CheckName;
use crate::commands::check_report::{Check, Outcome, Row};

const SHOWN: usize = 5;

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let unsettled = match machine.store() {
        Ok(Store::NoneYet) => return vec![settled()],
        Ok(Store::Read(read)) if read.version == SqliteStore::SCHEMA_VERSION => &read.unsettled,
        Ok(Store::Read(_)) => {
            return vec![not_checked("the database is not at this dam's version")];
        }
        Ok(Store::LeftOver(_)) => return vec![not_checked("the database was not read")],
        Err(_) => return vec![not_checked("the database did not open")],
    };
    match unsettled {
        Ok(unsettled) if unsettled.is_empty() => vec![settled()],
        Ok(unsettled) => vec![listed(unsettled)],
        Err(error) => vec![not_checked(&format!("the database did not read: {error}"))],
    }
}

fn settled() -> Check {
    Check::of(
        CheckName::Parents,
        Outcome::Ok,
        "dam knows every object's parent",
    )
}

fn not_checked(why: &str) -> Check {
    Check::of(
        CheckName::Parents,
        Outcome::NotRun,
        format!("not checked, {why}"),
    )
}

fn listed(unsettled: &[Unsettled]) -> Check {
    let objects = match unsettled.len() {
        1 => "1 object".to_string(),
        n => format!("{n} objects"),
    };
    let detail = format!("{objects} whose parent dam does not know");
    let mut rows: Vec<Row> = unsettled.iter().take(SHOWN).map(row).collect();
    if unsettled.len() > SHOWN {
        let more = format!("{} more", unsettled.len() - SHOWN);
        rows.push(Row::of(
            &more,
            Outcome::Warning,
            "shown once these are settled",
        ));
    }
    Check::of(CheckName::Parents, Outcome::Warning, detail).listing(rows)
}

fn row(unsettled: &Unsettled) -> Row {
    let base = unsettled.object.base();
    let oid = unsettled.object.oid().short();
    let could_hold: Vec<&str> = unsettled.could_hold.iter().map(|o| o.short()).collect();
    let parent = match could_hold.as_slice() {
        [only] => only.to_string(),
        _ => "<oid>".to_string(),
    };
    let detail = format!(
        "{} in {}, its parent is one of {}",
        base.subject,
        base.path,
        could_hold.join(", ")
    );
    let pull = if unsettled.pulled {
        "dam pull, or "
    } else {
        ""
    };
    Row::of(oid, Outcome::Warning, detail)
        .advising(format!("run: {pull}dam mv {oid} --parent {parent}"))
}
