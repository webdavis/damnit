mod agenda;
mod authorship;
mod caller_id;
pub(crate) mod catalogue;
mod commit;
pub(crate) mod config;
mod day;
mod done;
mod edit;
mod follow_ons;
mod ls;
mod mv;
mod new;
mod parsing;
mod reminders;
mod remote;
mod restore;
mod rm;
mod sent;
mod settled;
mod show;
mod stage;
mod status;
mod sync;
pub(crate) mod token;
mod token_warnings;

use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn pulls_a_stale_remote(command: &Command) -> bool {
    matches!(
        command,
        Command::Ls(_) | Command::Show(_) | Command::Agenda(_) | Command::Reminders(_)
    )
}

pub(crate) fn dispatch_a_verb_that_opens_the_store(
    ctx: &Context,
    command: Command,
) -> Result<Report, CliError> {
    crate::gate::admit(ctx.access.as_ref(), &command)?;
    match command {
        Command::New(a) => new::run(ctx, a),
        Command::Done(a) => done::run(ctx, a),
        Command::Edit(a) => edit::run(ctx, a),
        Command::Mv(a) => mv::run(ctx, a),
        Command::Rm(a) => rm::run(ctx, a),
        Command::Add(a) => stage::run_add(ctx, a),
        Command::Reset(a) => stage::run_reset(ctx, a),
        Command::Restore(a) => restore::run(ctx, a),
        Command::Commit(a) => commit::run_commit(ctx, a),
        Command::Log => commit::run_log(ctx),
        Command::Show(a) => show::run_show(ctx, a),
        Command::Status(a) => status::run_status(ctx, a),
        Command::Diff(a) => status::run_diff(ctx, a),
        Command::Ls(a) => ls::run_ls(ctx, a),
        Command::Agenda(a) => agenda::run_agenda(ctx, a),
        Command::Day(a) => day::run_day(ctx, a),
        Command::Reminders(a) => reminders::run_reminders(ctx, a),
        Command::Remote(a) => remote::run_remote(ctx, a),
        Command::Push(a) => sync::run_push(ctx, a),
        Command::Pull(a) => sync::run_pull(ctx, a),
        Command::Resolve(a) => sync::run_resolve(ctx, a),
        Command::Category(_) | Command::Filter(_) | Command::Token(_) | Command::Config(_) => {
            unreachable!("answered before the store")
        }
    }
}
