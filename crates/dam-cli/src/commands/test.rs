mod credentials;
#[cfg(test)]
mod tests;

use dam_adapters::LuaConfigFile;
use dam_application::HelperLauncher;

use crate::args::PartName;
use crate::commands::check_report::{self, Check, CheckReport, Heading};
use crate::error::CliError;
use crate::output::Format;

const PARTS: Heading = Heading {
    title: "Parts",
    blurb: "whether each part does its real job",
};

type Run = fn(&LuaConfigFile, &dyn HelperLauncher) -> Check;

const EVERY_PART: [(PartName, Run); 1] = [(PartName::Credentials, credentials::test)];

pub(crate) fn print(
    asked: &[PartName],
    config_file: &LuaConfigFile,
    launcher: &dyn HelperLauncher,
    format: Format,
    color: bool,
) -> Result<(), CliError> {
    check_report::print(report(asked, config_file, launcher), format, color)
}

fn report(
    asked: &[PartName],
    config_file: &LuaConfigFile,
    launcher: &dyn HelperLauncher,
) -> CheckReport {
    let checks = EVERY_PART
        .iter()
        .filter(|(name, _)| asked.is_empty() || asked.contains(name))
        .map(|(_, test)| test(config_file, launcher))
        .collect();
    CheckReport::new("test", PARTS, checks)
}
