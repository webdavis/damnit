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
    #[arg(
        required = true,
        help = "The objects to set back, named in full or by oid prefix"
    )]
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct CommitArgs {
    #[arg(short = 'm', long)]
    pub(crate) message: String,
}

#[derive(Args, Debug)]
pub(crate) struct ShowArgs {
    #[arg(help = "An object oid or a commit id, full or prefixed")]
    pub(crate) id: String,
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    #[arg(
        long,
        help = "Embed the whole before and after object in every change; shapes --json and --toon only"
    )]
    pub(crate) full: bool,
}

#[derive(Args, Debug)]
pub(crate) struct DiffArgs {
    #[arg(long)]
    pub(crate) staged: bool,
    #[arg(
        long,
        help = "Embed the whole before and after object in every change; shapes --json and --toon only"
    )]
    pub(crate) full: bool,
}
