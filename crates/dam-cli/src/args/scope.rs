use clap::Args;
use dam_domain::Path;

use super::values::path_arg;

#[derive(Args, Debug, Default)]
pub(crate) struct ScopeArgs {
    #[arg(
        long,
        conflicts_with = "project",
        help = "Reach every task, not only the project this folder is in; a path starts from the root"
    )]
    pub(crate) global: bool,
    #[arg(
        long,
        value_name = "PATH",
        value_parser = path_arg,
        help = "Work in the project at PATH, such as pns/hooks/, from any folder"
    )]
    pub(crate) project: Option<Path>,
}
