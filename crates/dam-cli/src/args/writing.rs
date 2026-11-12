use clap::Args;
use dam_domain::{ChildDisposition, DependencyDisposition, Path, Priority};

use super::dispositions::{child_disposition, dependency_disposition};
use super::values::{path_arg, priority_arg};

#[derive(Args, Debug)]
pub(crate) struct NewArgs {
    #[arg(help = "Title the new task or event")]
    pub(crate) subject: String,
    #[arg(
        long,
        help = "Create an event instead of a task, which needs --start and --end"
    )]
    pub(crate) event: bool,
    #[arg(
        long,
        default_value = "",
        hide_default_value = true,
        value_parser = path_arg,
        help = "Put it under PATH, such as work/clients/. Default: the root"
    )]
    pub(crate) path: Path,
    #[arg(
        long,
        help = "Set the due date: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM"
    )]
    pub(crate) due: Option<String>,
    #[arg(
        short = 'p',
        long,
        value_parser = priority_arg,
        help = "Set the priority, 1 (highest) to 4. Default: 4"
    )]
    pub(crate) priority: Option<Priority>,
    #[arg(
        long,
        help = "Set the deadline, a whole day read the way --due reads one"
    )]
    pub(crate) deadline: Option<String>,
    #[arg(long, alias = "label", help = "Add a label. Repeatable")]
    pub(crate) labels: Vec<String>,
    #[arg(
        long,
        default_value = "",
        hide_default_value = true,
        help = "Set the body text"
    )]
    pub(crate) body: String,
    #[arg(
        long,
        requires = "event",
        help = "Start the event at this time, read the way --due reads one"
    )]
    pub(crate) start: Option<String>,
    #[arg(
        long,
        requires = "event",
        help = "End the event at this time, read the way --due reads one"
    )]
    pub(crate) end: Option<String>,
}

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
