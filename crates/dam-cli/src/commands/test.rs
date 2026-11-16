mod credentials;
#[cfg(test)]
mod tests;

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

type Run = fn(&LuaConfigFile, &dyn HelperLauncher, &dyn Progress) -> Check;

const EVERY_PART: [(PartName, Run); 1] = [(PartName::Credentials, credentials::test)];

pub(crate) fn print(
    asked: &[PartName],
    config_file: &LuaConfigFile,
    launcher: &dyn HelperLauncher,
    progress: &dyn Progress,
    format: Format,
    color: bool,
) -> Result<(), CliError> {
    let report = report(asked, config_file, launcher, progress);
    check_report::print(report, format, color)
}

fn report(
    asked: &[PartName],
    config_file: &LuaConfigFile,
    launcher: &dyn HelperLauncher,
    progress: &dyn Progress,
) -> CheckReport {
    let checks = EVERY_PART
        .iter()
        .filter(|(name, _)| asked.is_empty() || asked.contains(name))
        .map(|(_, test)| test(config_file, launcher, progress))
        .collect();
    CheckReport::new("test", PARTS, checks)
}
