use serde::Serialize;

#[derive(clap::ValueEnum, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckName {
    Config,
}

impl CheckName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CheckName::Config => "config",
        }
    }
}
