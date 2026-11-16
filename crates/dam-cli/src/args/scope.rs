use clap::Args;

#[derive(Args, Debug, Default)]
pub(crate) struct ScopeArgs {
    #[arg(
        long,
        help = "Reach every task, not only the project this folder is in; a path starts from the root"
    )]
    pub(crate) global: bool,
}
