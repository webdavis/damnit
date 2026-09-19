use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "tasks and events, staged and committed like git"
)]
pub struct Cli {
    /// Print the result as JSON.
    #[arg(long, global = true, conflicts_with = "toon")]
    pub json: bool,
    /// Print the result as TOON, a compact form for language models.
    #[arg(long, global = true)]
    pub toon: bool,
    #[arg(long, global = true, env = "DAM_CONFIG", value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(long, global = true, env = "DAM_STORE", value_name = "FILE")]
    pub store: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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
pub struct NewArgs {
    pub subject: String,
    #[arg(long)]
    pub event: bool,
    #[arg(long, default_value = "")]
    pub path: String,
    #[arg(long)]
    pub due: Option<String>,
    #[arg(short = 'p', long)]
    pub priority: Option<u8>,
    #[arg(long)]
    pub deadline: Option<String>,
    #[arg(long = "label")]
    pub labels: Vec<String>,
    #[arg(long, default_value = "")]
    pub body: String,
    #[arg(long, requires = "event")]
    pub start: Option<String>,
    #[arg(long, requires = "event")]
    pub end: Option<String>,
}

#[derive(Args, Debug)]
pub struct DoneArgs {
    pub oid: String,
    #[arg(long)]
    pub force: bool,
    #[arg(long, requires = "force")]
    pub interactive: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    pub oid: String,
    /// Open the object in your editor instead of passing flags.
    #[arg(short = 'e', long)]
    pub editor: bool,
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(short = 'p', long)]
    pub priority: Option<u8>,
    #[arg(long, conflicts_with = "no_due")]
    pub due: Option<String>,
    #[arg(long)]
    pub no_due: bool,
    #[arg(long, conflicts_with = "no_deadline")]
    pub deadline: Option<String>,
    #[arg(long)]
    pub no_deadline: bool,
    #[arg(long = "label")]
    pub labels: Vec<String>,
    #[arg(long = "unlabel")]
    pub unlabels: Vec<String>,
    #[arg(long = "depends")]
    pub depends: Vec<String>,
    #[arg(long = "undepends")]
    pub undepends: Vec<String>,
    #[arg(long, conflicts_with = "no_recurrence")]
    pub recurrence: Option<String>,
    #[arg(long)]
    pub no_recurrence: bool,
    #[arg(long, conflicts_with = "detach")]
    pub attach: Option<String>,
    #[arg(long)]
    pub detach: bool,
    #[arg(long)]
    pub start: Option<String>,
    #[arg(long)]
    pub end: Option<String>,
    #[arg(long, conflicts_with = "no_location")]
    pub location: Option<String>,
    #[arg(long)]
    pub no_location: bool,
}

#[derive(Args, Debug)]
pub struct MvArgs {
    pub oid: String,
    pub path: String,
}

#[derive(Args, Debug)]
pub struct RmArgs {
    pub oid: String,
}

#[derive(Args, Debug)]
#[group(required = true)]
pub struct AddArgs {
    #[arg(conflicts_with = "all")]
    pub oids: Vec<String>,
    #[arg(short = 'A', long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct ResetArgs {
    pub oids: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CommitArgs {
    #[arg(short = 'm', long)]
    pub message: String,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// An object oid or a commit id, full or prefixed.
    pub id: String,
}

#[derive(Args, Debug)]
pub struct DiffArgs {
    #[arg(long)]
    pub staged: bool,
}

#[derive(Args, Debug)]
pub struct LsArgs {
    pub query: Option<String>,
}

#[derive(Args, Debug)]
pub struct RemoteArgs {
    #[command(subcommand)]
    pub command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
pub enum RemoteCommand {
    /// Add a remote to the config file.
    Add { name: String, url: String },
    /// List configured remotes.
    List,
}

#[derive(Args, Debug)]
pub struct PushArgs {
    pub remote: Option<String>,
}

#[derive(Args, Debug)]
pub struct PullArgs {
    pub remote: Option<String>,
}

#[derive(Args, Debug)]
pub struct ResolveArgs {
    pub oid: String,
    #[arg(long, conflicts_with = "theirs", required_unless_present = "theirs")]
    pub ours: bool,
    #[arg(long)]
    pub theirs: bool,
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
    fn add_all_and_oids_are_alternatives() {
        assert!(Cli::try_parse_from(["dam", "add", "-A"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "add", "abc", "def"]).is_ok());
        assert!(Cli::try_parse_from(["dam", "add"]).is_err());
    }
}
