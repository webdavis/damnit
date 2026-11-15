mod check;
mod report;
mod row;

pub(crate) use check::{Check, ConfigFiles, Outcome};
pub(crate) use report::{CheckReport, Heading};
pub(crate) use row::Row;

use crate::error::CliError;
use crate::output::{self, Format};
use crate::view;

pub(crate) fn print(report: CheckReport, format: Format, color: bool) -> Result<(), CliError> {
    output::print(format, color, &view::report(&report)?)?;
    match report.found_an_issue() {
        true => Err(CliError::Reported),
        false => Ok(()),
    }
}
