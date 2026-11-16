mod credentials;
pub(crate) mod filters;
#[cfg(test)]
mod tests;

use std::path::Path;

use dam_adapters::LuaConfigFile;
use dam_application::{HelperLauncher, Progress};

use crate::args::PartName;
use crate::commands::check_report::{self, Check, CheckReport, Heading};
use crate::error::CliError;
use crate::output::Format;

const PARTS: Heading = Heading {
    title: "Parts",
    blurb: "whether each part does its real job",
};

pub(crate) struct Machine<'a> {
    pub(crate) config_file: &'a LuaConfigFile,
    pub(crate) store_path: &'a Path,
    pub(crate) launcher: &'a dyn HelperLauncher,
    pub(crate) progress: &'a dyn Progress,
}

type Run = fn(&Machine) -> Check;

const EVERY_PART: [(PartName, Run); 2] = [
    (PartName::Credentials, |m| {
        credentials::test(m.config_file, m.launcher, m.progress)
    }),
    (PartName::Filters, |m| {
        filters::test(m.config_file, m.store_path, m.progress)
    }),
];

pub(crate) fn print(
    asked: &[PartName],
    machine: &Machine,
    format: Format,
    color: bool,
) -> Result<(), CliError> {
    check_report::print(report(asked, machine), format, color)
}

fn report(asked: &[PartName], machine: &Machine) -> CheckReport {
    let checks = EVERY_PART
        .iter()
        .filter(|(name, _)| asked.is_empty() || asked.contains(name))
        .map(|(_, test)| test(machine))
        .collect();
    CheckReport::new("test", PARTS, checks)
}
