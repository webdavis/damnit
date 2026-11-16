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
    #[arg(
        long,
        help = "Show what would be staged and which waiting items taken, and write nothing"
    )]
    pub(crate) preview: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ResetArgs {
    #[arg(
        help = "Unstage these objects, or put these declined items back to waiting, by oid, hash or prefix. Default: everything staged"
    )]
    pub(crate) oids: Vec<String>,
}

#[derive(Args, Debug)]
pub(crate) struct RestoreArgs {
    #[arg(required = true, help = "Set back these objects, by oid or prefix")]
    pub(crate) oids: Vec<String>,
    #[arg(long, help = "Show what would be thrown away and write nothing")]
    pub(crate) preview: bool,
}

#[derive(Args, Debug)]
pub(crate) struct RevertArgs {
    #[arg(help = "Pick the commit by id, full or prefixed")]
    pub(crate) commit: String,
    #[arg(help = "Set back only these objects' changes in the commit, by oid or prefix")]
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
        help = "Embed the whole object before and after each change, under -o"
    )]
    pub(crate) full: bool,
}

#[derive(Args, Debug)]
pub(crate) struct DiffArgs {
    #[arg(long, help = "Show the staged changes instead of the unstaged ones")]
    pub(crate) staged: bool,
    #[arg(
        long,
        help = "Embed the whole object before and after each change, under -o"
    )]
    pub(crate) full: bool,
}

pub(crate) const REVERT_HELP: &str =
    "Each field the commit changed goes back to its value before the commit.
A field changed again since is kept as it is and named.
A task the commit created is removed only when nothing about it changed since.
A task the commit removed comes back as it was.
It stages and commits nothing: read it with dam diff, then dam add and dam commit it.
It refuses a commit that dam pull made.

Examples:
  dam revert 1a2b3c4
  dam revert 1a2b3c4 5d6e7f8
  dam revert 1a2b3c4 -o json";
