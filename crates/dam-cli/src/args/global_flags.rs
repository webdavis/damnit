use std::path::PathBuf;

use clap::Parser;

use super::{Command, values};
use crate::output::Format;
use crate::progress::Asked;

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "Track tasks and events, staged and committed like git",
    max_term_width = 100
)]
pub(crate) struct Cli {
    #[arg(
        short,
        long,
        global = true,
        value_name = "FORMAT",
        value_parser = Format::chosen,
        help = "Print json, toon or yaml, and a failure as one error document in that format on stderr; toon is a compact form for language models; dam doctor, dam test and dam config validate print only their report, to stdout"
    )]
    pub(crate) output: Option<Format>,
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
        help = "Print without color; color is also off under NO_COLOR or when not a terminal"
    )]
    pub(crate) no_color: bool,
    #[arg(
        short,
        long,
        global = true,
        help = "Print only errors; -o still prints its answer"
    )]
    pub(crate) quiet: bool,
    #[arg(
        long,
        global = true,
        conflicts_with_all = ["no_progress", "quiet"],
        help = "Show the progress of slow steps on stderr, even when it is not a terminal"
    )]
    pub(crate) progress: bool,
    #[arg(long, global = true, help = "Hide the progress of slow steps")]
    pub(crate) no_progress: bool,
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

impl Cli {
    pub(crate) fn progress_asked(&self) -> Asked {
        Asked {
            setting: true,
            progress: self.progress,
            no_progress: self.no_progress,
            quiet: self.quiet,
        }
    }
}
