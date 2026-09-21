//! Reading the working layer by query or by saved filter.

use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct LsArgs {
    pub(crate) query: Option<String>,
}
