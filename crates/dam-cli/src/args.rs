use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "tasks and events, staged and committed like git"
)]
pub(crate) struct Cli {
    /// Print the result as JSON.
    #[arg(long, global = true, conflicts_with = "toon")]
    pub(crate) json: bool,
    /// Print the result as TOON, a compact form for language models.
    #[arg(long, global = true)]
    pub(crate) toon: bool,
    #[arg(long, global = true, env = "DAM_CONFIG", value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,
    #[arg(long, global = true, env = "DAM_STORE", value_name = "FILE")]
    pub(crate) store: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Create a task, or an event with --event.
    New(NewArgs),
    /// Mark a task done.
    Done(DoneArgs),
    /// Change fields on an object.
    Edit(EditArgs),
    /// Move an object and its children to another path.
    Mv(MvArgs),
    /// Remove an object from the working layer.
    Rm(RmArgs),
    /// Stage changes.
    Add(AddArgs),
    /// Unstage changes.
    Reset(ResetArgs),
    /// Set objects back to their last committed state, staged or not.
    Restore(RestoreArgs),
    /// Record the stage as a commit.
    Commit(CommitArgs),
    /// List commits, newest first.
    Log,
    /// Show one object or one commit.
    Show(ShowArgs),
    /// Working versus stage versus last commit, plus remote notices.
    Status,
    /// Unstaged changes, or staged with --staged.
    Diff(DiffArgs),
    /// List objects, optionally by query or saved filter.
    Ls(LsArgs),
    /// Manage remotes.
    Remote(RemoteArgs),
    /// Send unpushed commits to a remote, or to every remote.
    Push(PushArgs),
    /// Fetch and merge from a remote, or from every remote.
    Pull(PullArgs),
    /// Settle a conflict for one object.
    Resolve(ResolveArgs),
}

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
    #[arg(long)]
    pub(crate) force: bool,
    #[arg(long, requires = "force")]
    pub(crate) interactive: bool,
}

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
pub(crate) struct DiffArgs {
    #[arg(long)]
    pub(crate) staged: bool,
}

#[derive(Args, Debug)]
pub(crate) struct LsArgs {
    pub(crate) query: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct RemoteArgs {
    #[command(subcommand)]
    pub(crate) command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RemoteCommand {
    /// Add a remote to the config file.
    Add { name: String, url: String },
    /// List configured remotes.
    List,
}

#[derive(Args, Debug)]
pub(crate) struct PushArgs {
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct PullArgs {
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct ResolveArgs {
    pub(crate) oid: String,
    #[arg(long, conflicts_with = "theirs", required_unless_present = "theirs")]
    pub(crate) ours: bool,
    #[arg(long)]
    pub(crate) theirs: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn new_with_flags_parses() {
        let cli = Cli::try_parse_from([
            "dam", "new", "buy milk", "--path", "inbox/", "--due", "tomorrow", "-p", "1",
            "--label", "errand",
        ])
        .unwrap();
        match cli.command {
            Command::New(a) => {
                assert_eq!(a.subject, "buy milk");
                assert_eq!(a.path, "inbox/");
                assert_eq!(a.due.as_deref(), Some("tomorrow"));
                assert_eq!(a.priority, Some(1));
                assert_eq!(a.labels, vec!["errand".to_string()]);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn json_and_toon_are_global_and_exclusive() {
        let cli = Cli::try_parse_from(["dam", "ls", "--json"]).unwrap();
        assert!(cli.json);
        assert!(Cli::try_parse_from(["dam", "ls", "--json", "--toon"]).is_err());
    }

    #[test]
    fn resolve_needs_exactly_one_side() {
        assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "resolve", "abc"]).is_err());
        assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours", "--theirs"]).is_err());
    }

    #[test]
    fn edit_takes_undone_beside_the_other_clearing_flags() {
        let cli = Cli::try_parse_from(["dam", "edit", "abcd", "--undone"]).unwrap();
        match cli.command {
            Command::Edit(a) => assert!(a.undone),
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn restore_needs_at_least_one_oid() {
        assert!(Cli::try_parse_from(["dam", "restore", "abcd"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "restore"]).is_err());
    }

    #[test]
    fn add_all_and_oids_are_alternatives() {
        assert!(Cli::try_parse_from(["dam", "add", "-A"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "add", "abc", "def"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "add"]).is_err());
    }
}
