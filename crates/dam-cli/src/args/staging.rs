use clap::Args;

#[derive(Args, Debug)]
#[group(required = true)]
pub(crate) struct AddArgs {
    #[arg(
        conflicts_with = "all",
        help = "Stage the changes to these objects, by oid or prefix"
    )]
    pub(crate) oids: Vec<String>,
    #[arg(short = 'A', long, help = "Stage every change")]
    pub(crate) all: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ResetArgs {
    #[arg(help = "Unstage these objects, by oid or prefix. Default: everything staged")]
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct RestoreArgs {
    #[arg(required = true, help = "Set back these objects, by oid or prefix")]
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct CommitArgs {
    #[arg(short = 'm', long, help = "Describe the commit")]
    pub(crate) message: String,
}

#[derive(Args, Debug)]
pub(crate) struct ShowArgs {
    #[arg(help = "Pick an object by oid or a commit by id, full or prefixed")]
    pub(crate) id: String,
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    #[arg(
        long,
        help = "Embed the whole object before and after each change, in --json and --toon"
    )]
    pub(crate) full: bool,
}

#[derive(Args, Debug)]
pub(crate) struct DiffArgs {
    #[arg(long, help = "Show the staged changes instead of the unstaged ones")]
    pub(crate) staged: bool,
    #[arg(
        long,
        help = "Embed the whole object before and after each change, in --json and --toon"
    )]
    pub(crate) full: bool,
}
