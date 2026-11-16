use clap::{Args, Subcommand};

use super::values::label_arg;

#[derive(Args, Debug)]
pub(crate) struct TokenArgs {
    #[command(subcommand)]
    pub(crate) command: TokenCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TokenCommand {
    #[command(about = "Make a token with one grant, add it to secrets.lua and print it once")]
    Create(CreateArgs),
    #[command(
        about = "List the tokens in secrets.lua with their grants and standing, never their values"
    )]
    List,
    #[command(about = "Remove a token's table from secrets.lua")]
    Revoke {
        #[arg(help = "Name the token to remove")]
        name: String,
    },
}

#[derive(Args, Debug)]
pub(crate) struct CreateArgs {
    #[arg(help = "Name the token, such as tool")]
    pub(crate) name: String,
    #[arg(
        long,
        required = true,
        value_name = "PERMISSION",
        help = "Allow a permission, such as task:read. Repeatable"
    )]
    pub(crate) allow: Vec<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Narrow the grant to objects under a path. Repeatable"
    )]
    pub(crate) paths: Vec<String>,
    #[arg(
        long,
        value_name = "LABEL",
        value_delimiter = ',',
        value_parser = label_arg,
        help = "Narrow the grant to objects carrying a label, such as bug,urgent. Repeatable"
    )]
    pub(crate) labels: Vec<String>,
    #[arg(long, help = "Narrow the grant to objects the token made")]
    pub(crate) own: bool,
    #[arg(
        long,
        value_name = "REMOTE",
        help = "Narrow the grant to a remote. Repeatable"
    )]
    pub(crate) remotes: Vec<String>,
    #[arg(
        long,
        value_name = "INSTANT",
        help = "Stop the token working at a date-time with an offset, such as 2027-05-11T00:00:00Z"
    )]
    pub(crate) expires: Option<String>,
    #[arg(
        long,
        value_name = "CHANNEL",
        help = "Let the token work over local or mcp. Repeatable. Default: local"
    )]
    pub(crate) via: Vec<String>,
    #[arg(
        long,
        value_name = "KEY",
        help = "Refuse a request that does not pass --meta KEY=..., such as session. Repeatable"
    )]
    pub(crate) require: Vec<String>,
}
