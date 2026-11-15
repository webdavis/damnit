use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommand {
    #[command(
        about = "List where the config files are and every setting dam is using, its own defaults marked, never a secret",
        after_help = "It runs the top-level code of secrets.lua and calls no credential function.\n\nExamples:\n  dam config list\n  dam config list --json"
    )]
    List,
    #[command(
        about = "Print one setting, or every setting in one part, by its key",
        after_help = "It runs the top-level code of secrets.lua and calls no credential function.\n\nExamples:\n  dam config get remote.todoist.stale\n  dam config get remote.todoist\n  dam config get remote.todoist --json"
    )]
    Get {
        #[arg(
            help = "Name the setting by the key dam config list shows, such as remote.todoist.stale"
        )]
        key: String,
    },
    #[command(
        about = "Print the starter config.lua a first run writes, without writing a file",
        after_help = "Examples:\n  dam config print-default\n  dam config print-default > config.lua"
    )]
    PrintDefault,
    #[command(
        about = "Check the config as dam doctor does; an error raised in secrets.lua is shown as that file wrote it",
        after_help = "Examples:\n  dam config validate\n  dam config validate --json"
    )]
    Validate,
}
