use std::cmp::Ordering;
use std::path::PathBuf;

use dam_adapters::SqliteStore;

use super::check::{Check, Outcome};
use super::machine::Machine;
use super::store_reading::Store;
use crate::args::CheckName;

const SUPPORTED: u32 = SqliteStore::SCHEMA_VERSION;
const SETTLE: &str =
    "run dam status, which opens the store and settles what is left, then run dam doctor";
const RESTORE: &str = "restore it from a backup, or point DAM_STORE at another file";

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let found = |result, detail: String| Check::of(CheckName::Database, result, detail);
    let check = match machine.store() {
        Ok(Store::NoneYet) => found(Outcome::Ok, "none yet".into()),
        Ok(Store::LeftOver(files)) => found(Outcome::Warning, left_over(files)).advising(SETTLE),
        Ok(Store::Read(opened)) => match opened.version.cmp(&SUPPORTED) {
            Ordering::Equal => found(Outcome::Ok, format!("version {SUPPORTED}")),
            Ordering::Less => found(
                Outcome::Warning,
                format!(
                    "version {}, the next dam command upgrades it to {SUPPORTED}",
                    opened.version
                ),
            ),
            Ordering::Greater => found(
                Outcome::Issue,
                format!(
                    "version {}, newer than the {SUPPORTED} this dam reads",
                    opened.version
                ),
            )
            .advising("upgrade dam"),
        },
        Err(error) => found(
            Outcome::Issue,
            format!("{} does not open: {error}", machine.store_path.display()),
        )
        .advising(RESTORE),
    };
    vec![check]
}

fn left_over(files: &[PathBuf]) -> String {
    let names: Vec<String> = files
        .iter()
        .filter_map(|file| file.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    let verb = if names.len() == 1 { "is" } else { "are" };
    format!(
        "not read, {} {verb} left beside it, so the file alone may read wrong",
        names.join(" and ")
    )
}
