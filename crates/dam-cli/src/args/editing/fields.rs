use clap::Args;

use crate::args::values::estimate_arg;

#[derive(Args, Debug, Default)]
pub(crate) struct FieldArgs {
    #[arg(
        long = "depends",
        help = "Make it wait on the task with this oid. Repeatable"
    )]
    pub(crate) depends: Vec<String>,
    #[arg(
        long,
        help = "Repeat the task by a rule such as \"every 2 weeks on mon\""
    )]
    pub(crate) recurrence: Option<String>,
    #[arg(
        long,
        alias = "attach",
        help = "Attach the task to the event with this oid"
    )]
    pub(crate) attached: Option<String>,
    #[arg(long, help = "Set the event's location")]
    pub(crate) location: Option<String>,
    #[arg(
        long,
        value_parser = estimate_arg,
        help = "Set how long the task should take: minutes, such as 5, or 5m or 2h"
    )]
    pub(crate) estimate: Option<u32>,
    #[arg(
        long,
        value_name = "DAY",
        help = "Mark the task to work on first on DAY, after the ones marked before it. 3 a day"
    )]
    pub(crate) focus: Option<String>,
}

impl FieldArgs {
    pub(crate) fn task_only_flag(&self) -> Option<&'static str> {
        [
            (self.recurrence.is_some(), "--recurrence"),
            (self.attached.is_some(), "--attached"),
            (self.estimate.is_some(), "--estimate"),
            (self.focus.is_some(), "--focus"),
        ]
        .into_iter()
        .find_map(|(given, flag)| given.then_some(flag))
    }
}
