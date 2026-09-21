//! The stage, the history, and the two reads over them.

use clap::Args;

#[derive(Args, Debug)]
#[group(required = true)]
pub(crate) struct AddArgs {
    #[arg(conflicts_with = "all")]
    pub(crate) oids: Vec<String>,
    #[arg(short = 'A', long)]
    pub(crate) all: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ResetArgs {
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct RestoreArgs {
    /// The objects to set back, named in full or by oid prefix.
    #[arg(required = true)]
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct CommitArgs {
    #[arg(short = 'm', long)]
    pub(crate) message: String,
}

#[derive(Args, Debug)]
pub(crate) struct ShowArgs {
    /// An object oid or a commit id, full or prefixed.
    pub(crate) id: String,
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    /// Embed the whole before and after object in every change.
    #[arg(long)]
    pub(crate) full: bool,
}

#[derive(Args, Debug)]
pub(crate) struct DiffArgs {
    #[arg(long)]
    pub(crate) staged: bool,
    /// Embed the whole before and after object in every change.
    #[arg(long)]
    pub(crate) full: bool,
}
