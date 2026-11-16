use dam_application::ListedHookRun;
use serde::Serialize;

use crate::args::hook::HookRunArgs;
use crate::commands::hook_error_row::HookErrorRow;
use crate::commands::hooked::hooked;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct HookRun {
    hook: &'static str,
    ran: usize,
    changed: usize,
    failed: usize,
    not_reached: usize,
    error: Option<HookErrorRow>,
}

impl HookRun {
    fn of(run: &ListedHookRun) -> HookRun {
        HookRun {
            hook: run.hook.as_str(),
            ran: run.ran,
            changed: run.changed,
            failed: run.failed,
            not_reached: run.not_reached,
            error: run.error.as_ref().map(HookErrorRow::of),
        }
    }
}

impl View for HookRun {
    fn human(&self) -> Human {
        Human::Lines(vec![format!(
            "{}: {} ran, {} changed, {} failed, {} not reached",
            self.hook, self.ran, self.changed, self.failed, self.not_reached
        )])
    }
}

pub(super) fn run(ctx: &Context, args: HookRunArgs) -> Result<Report, CliError> {
    let hook = args.hook.into();
    let run = ctx
        .dam()
        .run_hook_over_listed(hook, args.query.as_deref())?;
    report(&hooked(ctx, HookRun::of(&run)))
}

#[cfg(test)]
mod tests;
