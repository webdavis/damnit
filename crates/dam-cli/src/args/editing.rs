use clap::Args;
use dam_domain::Priority;

use super::values::{label_arg, priority_arg};
use super::{ScopeArgs, SetArgs};
pub(crate) use fields::FieldArgs;

mod fields;

#[derive(Args, Debug)]
pub(crate) struct EditArgs {
    #[arg(
        help = "Pick the objects by oid, by a prefix of 4 or more hex characters, or - to read the ids dam list prints from stdin",
        required_unless_present_any = ["filter"]
    )]
    pub(crate) oid: Vec<String>,
    #[command(flatten)]
    pub(crate) set: SetArgs,
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
        help = "Set the due date: a date word, today+N, now+LENGTH, YYYY-MM-DD or YYYY-MM-DDTHH:MM"
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
    #[arg(long, alias = "undone", help = "Reopen a completed task")]
    pub(crate) no_done: bool,
    #[arg(
        long,
        alias = "label",
        value_delimiter = ',',
        value_parser = label_arg,
        help = "Add labels, such as bug,urgent. Repeatable"
    )]
    pub(crate) labels: Vec<String>,
    #[arg(
        long,
        alias = "unlabel",
        value_delimiter = ',',
        value_parser = label_arg,
        help = "Remove labels, such as bug,urgent. Repeatable"
    )]
    pub(crate) unlabels: Vec<String>,
    #[arg(
        long = "undepends",
        help = "Stop it waiting on the task with this oid. Repeatable"
    )]
    pub(crate) undepends: Vec<String>,
    #[command(flatten)]
    pub(crate) fields: FieldArgs,
    #[arg(long, conflicts_with = "recurrence", help = "Stop the task repeating")]
    pub(crate) no_recurrence: bool,
    #[arg(
        long,
        alias = "detach",
        conflicts_with = "attached",
        help = "Detach the task from its event"
    )]
    pub(crate) no_attached: bool,
    #[arg(long, help = "Move the event's start, read the way --due reads one")]
    pub(crate) start: Option<String>,
    #[arg(long, help = "Move the event's end, read the way --due reads one")]
    pub(crate) end: Option<String>,
    #[arg(long, conflicts_with = "location", help = "Clear the event's location")]
    pub(crate) no_location: bool,
    #[arg(long, conflicts_with = "estimate", help = "Clear the task's estimate")]
    pub(crate) no_estimate: bool,
    #[arg(long, conflicts_with = "focus", help = "Clear the task's focus mark")]
    pub(crate) no_focus: bool,
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
}
