use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct HookArgs {
    #[command(subcommand)]
    pub(crate) command: Option<HookCommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum HookCommand {
    #[command(
        about = "List the hooks in the hooks folder, each with its file, setting and whether it loads"
    )]
    List,
}
