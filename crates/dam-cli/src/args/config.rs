use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommand {
    #[command(
        about = "Print the starter config.lua a first run writes, without writing a file",
        after_help = "Examples:\n  dam config print-default\n  dam config print-default > config.lua"
    )]
    PrintDefault,
    #[command(
        about = "Check the config as dam doctor does; an error's message may show a value from the file",
        after_help = "Examples:\n  dam config validate\n  dam config validate --json"
    )]
    Validate,
}
