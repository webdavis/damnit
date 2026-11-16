use clap::Args;
use dam_domain::Path;

use super::values::path_arg;

#[derive(Args, Debug)]
pub(crate) struct InitArgs {
    #[arg(
        long,
        value_parser = path_arg,
        help = "Keep this project's tasks under PATH. Default: the folder's own name"
    )]
    pub(crate) path: Option<Path>,
}

pub(crate) const INIT_HELP: &str =
    "It writes .dam.lua, which says where this project's tasks go, and creates
the task that holds that path as a working change when no object holds it yet.
Where .dam.lua exists, as in a clone, it creates what is missing: the project's task and each
section the file names. When nothing is missing it changes nothing.
It refuses inside another project's folder.

Examples:
  dam init
  dam init --path work/app";
