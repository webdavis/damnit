use clap::Args;
use dam_domain::{CallerId, ChildDisposition, DependencyDisposition, Path};

use super::dispositions::{child_disposition, dependency_disposition};
use super::values::{caller_id_arg, path_arg};
use super::{ScopeArgs, SetArgs};

#[derive(Args, Debug)]
pub(crate) struct DoneArgs {
    #[arg(
        help = "Pick the tasks by oid, or by a prefix of 4 or more hex characters",
        required_unless_present_any = ["query", "filter"]
    )]
    pub(crate) oid: Vec<String>,
    #[command(flatten)]
    pub(crate) set: SetArgs,
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
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    #[arg(skip)]
    pub(crate) project_children: Option<(Path, ChildDisposition)>,
}

#[derive(Args, Debug)]
pub(crate) struct MvArgs {
    #[arg(
        value_name = "OID... PATH",
        help = "Pick the objects by oid, or by a prefix of 4 or more hex characters, then the PATH to move them under, such as work/clients/",
        required_unless_present_any = ["query", "filter"]
    )]
    pub(crate) oid: Vec<String>,
    #[command(flatten)]
    pub(crate) set: SetArgs,
    #[arg(long, value_parser = path_arg, help = "Move them under PATH instead of naming it last")]
    pub(crate) path: Option<Path>,
    #[arg(long, help = "Put it in the object with this oid or prefix")]
    pub(crate) parent: Option<String>,
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
}

#[derive(Args, Debug)]
pub(crate) struct RmArgs {
    #[arg(
        help = "Pick the objects by oid, or by a prefix of 4 or more hex characters",
        required_unless_present_any = ["query", "filter", "id"]
    )]
    pub(crate) oid: Vec<String>,
    #[command(flatten)]
    pub(crate) set: SetArgs,
    #[arg(
        long,
        conflicts_with_all = ["oid", "query", "filter", "preview", "expect"],
        value_parser = caller_id_arg,
        help = "Take back the task or event you gave this id with dam new --id in the same session"
    )]
    pub(crate) id: Option<CallerId>,
}
