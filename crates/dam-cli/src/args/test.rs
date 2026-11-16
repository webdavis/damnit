use clap::Args;

pub(crate) const TEST_HELP: &str =
    "It contacts each remote, and runs credential functions, which may prompt.
The filters part runs each saved filter once over the store.
It writes nothing: not the store, not the config, not a remote.
It exits 1 when a part finds an issue, and prints the report to stdout either way.

Examples:
  dam test
  dam test credentials
  dam test filters
  dam test -o json";

#[derive(Args, Debug)]
pub(crate) struct TestArgs {
    #[arg(value_enum, help = "Test only these parts. Default: every part")]
    pub(crate) parts: Vec<PartName>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PartName {
    Credentials,
    Filters,
}
