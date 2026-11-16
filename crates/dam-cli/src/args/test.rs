use clap::Args;

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
