use std::path::PathBuf;

use clap::Parser;

use super::{Command, values};

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "Track tasks and events, staged and committed like git",
    max_term_width = 100
)]
pub(crate) struct Cli {
    #[arg(
        long,
        global = true,
        conflicts_with = "toon",
        help = "Print JSON, and a failure as one error document on stderr"
    )]
    pub(crate) json: bool,
    #[arg(
        long,
        global = true,
        help = "Print TOON, a compact form for language models; errors as --json"
    )]
    pub(crate) toon: bool,
    #[arg(
        long,
        global = true,
        env = "DAM_CONFIG",
        hide_env_values = true,
        value_name = "FILE",
        help = "Read the config from FILE; secrets.lua beside it holds the secrets. Default: ${XDG_CONFIG_HOME:-~/.config}/dam/config.lua"
    )]
    pub(crate) config: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        env = "DAM_STORE",
        hide_env_values = true,
        value_name = "FILE",
        help = "Open the store at FILE. Default: ${XDG_DATA_HOME:-~/.local/share}/dam/dam.db"
    )]
    pub(crate) store: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Never pull a stale remote first; answer from the local store"
    )]
    pub(crate) no_pull: bool,
    #[arg(
        long,
        global = true,
        help = "Print without color; color is also off under NO_COLOR or when not a terminal"
    )]
    pub(crate) no_color: bool,
    #[arg(
        long,
        global = true,
        value_name = "KEY=VALUE",
        value_parser = values::meta_arg,
        help = "State a detail of this request, such as session=abc123, kept with each change it makes. Repeatable"
    )]
    pub(crate) meta: Vec<(String, String)>,
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        help = "Read the request from FILE, or stdin when FILE is -, as JSON or TOON keyed by the command's flags"
    )]
    pub(crate) request: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}
