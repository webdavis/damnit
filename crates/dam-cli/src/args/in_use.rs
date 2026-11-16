use clap::{Args, Subcommand};
use dam_domain::Path;

use super::ScopeArgs;

#[derive(Args, Debug, Default)]
pub(crate) struct InUseArgs {
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    #[arg(skip)]
    pub(crate) place: Path,
}

#[derive(Args, Debug)]
pub(crate) struct LabelArgs {
    #[command(subcommand)]
    pub(crate) command: LabelCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum LabelCommand {
    #[command(about = "List every label the tasks and events carry, with how many carry each")]
    List(InUseArgs),
}

#[derive(Args, Debug)]
pub(crate) struct PathArgs {
    #[command(subcommand)]
    pub(crate) command: PathCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PathCommand {
    #[command(
        about = "List every path that holds a task or event, and each parent path, with how many sit at or under it"
    )]
    List(InUseArgs),
}
