//! The verbs that change an object: new, done, edit, mv and rm.

use clap::Args;
use dam_domain::{ChildDisposition, DependencyDisposition};

use super::dispositions::{child_disposition, dependency_disposition};

#[derive(Args, Debug)]
pub(crate) struct NewArgs {
    pub(crate) subject: String,
    #[arg(long)]
    pub(crate) event: bool,
    #[arg(long, default_value = "")]
    pub(crate) path: String,
    #[arg(long)]
    pub(crate) due: Option<String>,
    #[arg(short = 'p', long)]
    pub(crate) priority: Option<u8>,
    #[arg(long)]
    pub(crate) deadline: Option<String>,
    #[arg(long = "label")]
    pub(crate) labels: Vec<String>,
    #[arg(long, default_value = "")]
    pub(crate) body: String,
    #[arg(long, requires = "event")]
    pub(crate) start: Option<String>,
    #[arg(long, requires = "event")]
    pub(crate) end: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct DoneArgs {
    pub(crate) oid: String,
    /// Complete it even though something it waits on is still open.
    #[arg(long)]
    pub(crate) force: bool,
    /// Ask what happens to the open children and dependencies.
    #[arg(long, requires = "force")]
    pub(crate) interactive: bool,
    /// What happens to open children, instead of asking. Default: keep.
    #[arg(
        long,
        requires = "force",
        value_name = "up|keep|into:<name>",
        value_parser = child_disposition
    )]
    pub(crate) children: Option<ChildDisposition>,
    /// What happens to open dependency links, instead of asking. Default: keep.
    #[arg(
        long,
        requires = "force",
        value_name = "drop|keep",
        value_parser = dependency_disposition
    )]
    pub(crate) depends: Option<DependencyDisposition>,
}

/// `--children`: the words for the three answers the prompt offers, with the

#[derive(Args, Debug)]
pub(crate) struct EditArgs {
    pub(crate) oid: String,
    /// Open the object in your editor instead of passing flags.
    #[arg(short = 'e', long)]
    pub(crate) editor: bool,
    #[arg(long)]
    pub(crate) subject: Option<String>,
    #[arg(long)]
    pub(crate) body: Option<String>,
    #[arg(short = 'p', long)]
    pub(crate) priority: Option<u8>,
    #[arg(long, conflicts_with = "no_due")]
    pub(crate) due: Option<String>,
    #[arg(long)]
    pub(crate) no_due: bool,
    #[arg(long, conflicts_with = "no_deadline")]
    pub(crate) deadline: Option<String>,
    #[arg(long)]
    pub(crate) no_deadline: bool,
    /// Reopen a completed task.
    #[arg(long)]
    pub(crate) undone: bool,
    #[arg(long = "label")]
    pub(crate) labels: Vec<String>,
    #[arg(long = "unlabel")]
    pub(crate) unlabels: Vec<String>,
    #[arg(long = "depends")]
    pub(crate) depends: Vec<String>,
    #[arg(long = "undepends")]
    pub(crate) undepends: Vec<String>,
    #[arg(long, conflicts_with = "no_recurrence")]
    pub(crate) recurrence: Option<String>,
    #[arg(long)]
    pub(crate) no_recurrence: bool,
    #[arg(long, conflicts_with = "detach")]
    pub(crate) attach: Option<String>,
    #[arg(long)]
    pub(crate) detach: bool,
    #[arg(long)]
    pub(crate) start: Option<String>,
    #[arg(long)]
    pub(crate) end: Option<String>,
    #[arg(long, conflicts_with = "no_location")]
    pub(crate) location: Option<String>,
    #[arg(long)]
    pub(crate) no_location: bool,
}

#[derive(Args, Debug)]
pub(crate) struct MvArgs {
    pub(crate) oid: String,
    pub(crate) path: String,
}

#[derive(Args, Debug)]
pub(crate) struct RmArgs {
    pub(crate) oid: String,
}
