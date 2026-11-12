use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct TokenArgs {
    #[command(subcommand)]
    pub(crate) command: TokenCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TokenCommand {
    #[command(about = "Make a token with one grant, add it to the config and print it once")]
    Create(CreateArgs),
    #[command(
        about = "List the config's tokens with their grants and standing, never their values"
    )]
    List,
}

#[derive(Args, Debug)]
pub(crate) struct CreateArgs {
    #[arg(help = "Name the token, such as vpt")]
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
        help = "Narrow the grant to objects carrying a label. Repeatable"
    )]
    pub(crate) labels: Vec<String>,
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
}
