mod category;
mod category_written;
mod entry;
mod filter;
mod filter_written;
mod migrate;
mod migrated;

use dam_application::SetRun;

use crate::args::{CategoryCommand, FilterCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(super) fn run_category(ctx: &Context, command: CategoryCommand) -> Result<Report, CliError> {
    let file = ctx.settings_file.as_ref();
    match command {
        CategoryCommand::Create {
            name,
            values,
            exclusive,
        } => category::create(&ctx.config, file, &name, &values, exclusive),
        CategoryCommand::Remove { name } => {
            category::remove(&ctx.config, ctx.store.as_ref(), file, &name)
        }
        CategoryCommand::Migrate {
            map,
            preview,
            expect,
            update_config,
        } => migrate::migrate(
            &ctx.dam(),
            &ctx.config,
            file,
            migrate::Migration {
                map: &map,
                run: SetRun { preview, expect },
                update_config,
            },
        ),
        CategoryCommand::List => unreachable!("dam category list reads the config alone"),
    }
}

pub(super) fn run_filter(ctx: &Context, command: FilterCommand) -> Result<Report, CliError> {
    let file = ctx.settings_file.as_ref();
    match command {
        FilterCommand::Create { name, query } => filter::create(&ctx.config, file, &name, &query),
        FilterCommand::Remove { name } => filter::remove(&ctx.config, file, &name),
        FilterCommand::List => unreachable!("dam filter list reads the config alone"),
    }
}

#[cfg(test)]
mod category_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod migrate_refusal_tests;
#[cfg(test)]
mod migrate_tests;
