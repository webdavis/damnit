use clap::Args;
use dam_domain::{Kind, Path, Priority};

use super::values::{kind_arg, path_arg, priority_arg};

#[derive(Args, Debug)]
pub(crate) struct NewArgs {
    #[arg(help = "Title the new task or event")]
    pub(crate) subject: String,
    #[arg(
        long,
        default_value = "task",
        hide_default_value = true,
        default_value_if("event", "true", Some("event")),
        value_name = "task|event",
        value_parser = kind_arg,
        help = "Create a task or an event; an event needs --start and --end. Default: task"
    )]
    pub(crate) kind: Kind,
    #[arg(long, hide = true, conflicts_with = "kind")]
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
        help = "Start the event at this time, read the way --due reads one"
    )]
    pub(crate) start: Option<String>,
    #[arg(
        long,
        help = "End the event at this time, read the way --due reads one"
    )]
    pub(crate) end: Option<String>,
}
