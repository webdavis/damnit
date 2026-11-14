use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommand {
    #[command(
        about = "Read config.lua and secrets.lua in full and print where they are; an error's message may show a value from the file"
    )]
    Check,
}
