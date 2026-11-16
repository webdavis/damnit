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
        after_help = "It runs the top-level code of secrets.lua and calls no credential function.\nInside a project it also lists each setting from .dam.lua, marked as the project's.\n\nExamples:\n  dam config list\n  dam config list --json"
    )]
    List,
    #[command(
        about = "Print one setting, or every setting in one part, by its key",
        after_help = "It runs the top-level code of secrets.lua and calls no credential function.\n\nExamples:\n  dam config get remote.todoist.deadline\n  dam config get remote.todoist\n  dam config get remote.todoist --json"
    )]
    Get {
        #[arg(
            help = "Name the setting by the key dam config list shows, such as remote.todoist.deadline",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
        )]
        key: String,
    },
    #[command(
        about = "Write one setting into config.lua, or into .dam.lua inside a project, keeping the rest of the file",
        after_help = "A list takes its items separated by commas. A secret, a function or a hook is refused.\nInside a project, a key .dam.lua takes is written into .dam.lua, and a key only config.lua takes is refused; --global writes config.lua instead.\nInside a project, project.hooks is kept for this clone only, outside the repository; --global writes it into config.lua for every project.\n\nExamples:\n  dam config set remote.todoist.deadline 15m\n  dam config set category.priority.values high,low\n  dam config set project.hooks true\n  dam config set --global project.hooks true"
    )]
    Set {
        #[arg(
            help = "Name the setting by the key dam config list shows, such as remote.todoist.deadline"
        )]
        key: String,
        #[arg(help = "The value, read as that setting's type")]
        value: String,
        #[arg(
            long,
            help = "Write config.lua inside a project, not .dam.lua or this clone's project.hooks"
        )]
        global: bool,
    },
    #[command(
        about = "Remove one setting from config.lua, or from .dam.lua inside a project, keeping the rest of the file",
        after_help = "Inside a project, a key .dam.lua takes is removed from .dam.lua, except path, which a project needs; --global removes it from config.lua.\nInside a project, project.hooks forgets this clone's own switch; --global removes it from config.lua.

Examples:
  dam config unset remote.todoist.deadline
  dam config unset project.hooks"
    )]
    Unset {
        #[arg(
            help = "Name the setting by the key dam config list shows, such as remote.todoist.deadline"
        )]
        key: String,
        #[arg(
            long,
            help = "Remove the key from config.lua inside a project, not from .dam.lua or this clone's project.hooks"
        )]
        global: bool,
    },
    #[command(
        about = "Print the starter config.lua a first run writes, without writing a file",
        after_help = "Examples:\n  dam config print-default\n  dam config print-default > config.lua"
    )]
    PrintDefault,
    #[command(
        about = "Check the config as dam doctor config does; an error raised in secrets.lua is shown as that file wrote it",
        after_help = "It runs the top-level code of config.lua and secrets.lua and calls no credential function.\nIt exits 1 when the check finds an issue, and prints the report to stdout either way.\n\nExamples:\n  dam config validate\n  dam config validate --json"
    )]
    Validate,
}
