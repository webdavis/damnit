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
}
