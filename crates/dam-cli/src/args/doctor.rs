use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub(crate) struct DoctorArgs {
    #[arg(value_enum, help = "Run only these checks. Default: every check")]
    pub(crate) checks: Vec<CheckName>,
    #[arg(
        long,
        help = "Create the starter config and the store where they are missing"
    )]
    pub(crate) fix: bool,
}

#[derive(clap::ValueEnum, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckName {
    Config,
    Database,
    Helpers,
    Credentials,
    Pushes,
    Parents,
    Sync,
}

impl CheckName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CheckName::Config => "config",
            CheckName::Database => "database",
            CheckName::Helpers => "helpers",
            CheckName::Credentials => "credentials",
            CheckName::Pushes => "pushes",
            CheckName::Parents => "parents",
            CheckName::Sync => "sync",
        }
    }
}

pub(crate) const DOCTOR_HELP: &str =
    "An error raised in secrets.lua is shown as that file wrote it.
It runs the top-level code of config.lua and secrets.lua and calls no credential function.
It exits 1 when a check finds an issue, and prints the report to stdout either way.
--fix creates the starter config.lua and the store where each is missing, then runs the checks.
It changes no file that exists. With checks named, it creates only what those checks cover.

Examples:
  dam doctor
  dam doctor config
  dam doctor sync
  dam doctor --json
  dam doctor --fix
  dam doctor --fix database";
