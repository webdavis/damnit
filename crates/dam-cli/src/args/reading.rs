//! Reading the working layer by query or by saved filter, and the config
//! catalogue a client renders those filters and its label picker from.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct LsArgs {
    pub(crate) query: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct CategoryArgs {
    #[command(subcommand)]
    pub(crate) command: CategoryCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CategoryCommand {
    /// List declared categories with their values and whether they are exclusive.
    List,
}

#[derive(Args, Debug)]
pub(crate) struct FilterArgs {
    #[command(subcommand)]
    pub(crate) command: FilterCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum FilterCommand {
    /// List saved filters with their queries.
    List,
}
