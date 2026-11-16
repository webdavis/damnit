mod category;
mod category_written;
mod entry;
mod filter;
mod filter_written;
mod migrate;
mod migrated;
pub(crate) mod routed;

use dam_application::SetRun;

use crate::args::{CategoryCommand, FilterCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) fn run_category(
    ctx: &Context,
    command: CategoryCommand,
    standing: &Standing,
) -> Result<Report, CliError> {
    let file = ctx.settings_file.as_ref();
    match command {
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
                run: SetRun {
                    preview,
                    expect,
                    as_one: false,
                },
                update_config,
            },
        ),
        command => routed::category(&settings(ctx, standing), ctx.store.as_ref(), command),
    }
}

pub(crate) fn run_filter(
    ctx: &Context,
    command: FilterCommand,
    standing: &Standing,
) -> Result<Report, CliError> {
    routed::filter(&settings(ctx, standing), command)
}

fn settings<'a>(ctx: &'a Context, standing: &'a Standing) -> routed::Settings<'a> {
    routed::Settings {
        config: &ctx.config,
        personal: ctx.settings_file.as_ref(),
        standing,
    }
}

#[cfg(test)]
mod category_answer_tests;
#[cfg(test)]
mod category_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod migrate_refusal_tests;
#[cfg(test)]
mod migrate_tests;
#[cfg(test)]
mod project_tests;
