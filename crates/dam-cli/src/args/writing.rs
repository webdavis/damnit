use clap::Args;
use dam_domain::{ChildDisposition, DependencyDisposition, Path};

use super::dispositions::{child_disposition, dependency_disposition};
use super::values::path_arg;

#[derive(Args, Debug)]
pub(crate) struct DoneArgs {
    #[arg(help = "Pick the task by oid, or by a prefix of 4 or more hex characters")]
    pub(crate) oid: String,
    #[arg(
        long,
        help = "Complete it even though something it waits on is still open"
    )]
    pub(crate) force: bool,
    #[arg(
        long,
        requires = "force",
        help = "Ask what happens to the open children and dependencies"
    )]
    pub(crate) interactive: bool,
    #[arg(
        long,
        requires = "force",
        conflicts_with = "interactive",
        value_name = "up|keep|into:<name>",
        value_parser = child_disposition,
        help = "Decide what happens to open children instead of asking. Default: keep"
    )]
    pub(crate) children: Option<ChildDisposition>,
    #[arg(
        long,
        requires = "force",
        conflicts_with = "interactive",
        value_name = "drop|keep",
        value_parser = dependency_disposition,
        help = "Decide what happens to open dependency links instead of asking. Default: keep"
    )]
    pub(crate) depends: Option<DependencyDisposition>,
}

#[derive(Args, Debug)]
pub(crate) struct MvArgs {
    #[arg(help = "Pick the object by oid, or by a prefix of 4 or more hex characters")]
    pub(crate) oid: String,
    #[arg(value_parser = path_arg, help = "Move it under PATH, such as work/clients/")]
    pub(crate) path: Path,
}

#[derive(Args, Debug)]
pub(crate) struct RmArgs {
    #[arg(help = "Pick the object by oid, or by a prefix of 4 or more hex characters")]
    pub(crate) oid: String,
}
