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
    #[arg(long = "label", help = "Add a label. Repeatable")]
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
pub(crate) struct EditArgs {
    #[arg(help = "Pick the object by oid, or by a prefix of 4 or more hex characters")]
    pub(crate) oid: String,
    #[arg(
        short = 'e',
        long,
        help = "Open the object in your editor instead of passing flags"
    )]
    pub(crate) editor: bool,
    #[arg(long, help = "Replace the subject")]
    pub(crate) subject: Option<String>,
    #[arg(long, help = "Replace the body text")]
    pub(crate) body: Option<String>,
    #[arg(
        short = 'p',
        long,
        value_parser = priority_arg,
        help = "Set the priority, 1 (highest) to 4"
    )]
    pub(crate) priority: Option<Priority>,
    #[arg(
        long,
        conflicts_with = "no_due",
        help = "Set the due date: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM"
    )]
    pub(crate) due: Option<String>,
    #[arg(long, help = "Clear the due date")]
    pub(crate) no_due: bool,
    #[arg(
        long,
        conflicts_with = "no_deadline",
        help = "Set the deadline, a whole day read the way --due reads one"
    )]
    pub(crate) deadline: Option<String>,
    #[arg(long, help = "Clear the deadline")]
    pub(crate) no_deadline: bool,
    #[arg(long, help = "Reopen a completed task")]
    pub(crate) undone: bool,
    #[arg(long = "label", help = "Add a label. Repeatable")]
    pub(crate) labels: Vec<String>,
    #[arg(long = "unlabel", help = "Remove a label. Repeatable")]
    pub(crate) unlabels: Vec<String>,
    #[arg(
        long = "depends",
        help = "Make it wait on the task with this oid. Repeatable"
    )]
    pub(crate) depends: Vec<String>,
    #[arg(
        long = "undepends",
        help = "Stop it waiting on the task with this oid. Repeatable"
    )]
    pub(crate) undepends: Vec<String>,
    #[arg(
        long,
        conflicts_with = "no_recurrence",
        help = "Repeat the task by a rule such as \"every 2 weeks on mon\""
    )]
    pub(crate) recurrence: Option<String>,
    #[arg(long, help = "Stop the task repeating")]
    pub(crate) no_recurrence: bool,
    #[arg(
        long,
        conflicts_with = "detach",
        help = "Attach the task to the event with this oid"
    )]
    pub(crate) attach: Option<String>,
    #[arg(long, help = "Detach the task from its event")]
    pub(crate) detach: bool,
    #[arg(long, help = "Move the event's start, read the way --due reads one")]
    pub(crate) start: Option<String>,
    #[arg(long, help = "Move the event's end, read the way --due reads one")]
    pub(crate) end: Option<String>,
    #[arg(
        long,
        conflicts_with = "no_location",
        help = "Set the event's location"
    )]
    pub(crate) location: Option<String>,
    #[arg(long, help = "Clear the event's location")]
    pub(crate) no_location: bool,
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
