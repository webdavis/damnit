use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub(crate) struct DoctorArgs {
    #[arg(value_enum, help = "Run only these checks. Default: every check")]
    pub(crate) checks: Vec<CheckName>,
}

#[derive(clap::ValueEnum, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckName {
    Config,
    Database,
}

impl CheckName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CheckName::Config => "config",
            CheckName::Database => "database",
        }
    }
}
