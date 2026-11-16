mod agenda;
mod authorship;
mod caller_id;
pub(crate) mod catalogue;
mod catalogue_write;
mod check_report;
mod commit;
pub(crate) mod config;
pub(crate) mod config_files;
mod day;
pub(crate) mod doctor;
mod done;
pub(crate) mod each;
pub(crate) mod edit;
mod follow_ons;
pub(crate) mod hook_error_row;
pub(crate) mod hook_list;
mod hook_run;
mod hooked;
pub(crate) mod init;
mod ls;
mod mv;
mod new;
mod parsing;
mod reminders;
mod remote;
mod restore;
mod revert;
mod rm;
pub(crate) mod scoped;
mod sent;
mod settled;
mod show;
mod stage;
mod status;
mod sync;
pub(crate) mod test;
pub(crate) mod token;
mod token_warnings;

use crate::args::Command;
use crate::args::hook::{HookArgs, HookCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn dispatch_a_verb_that_opens_the_store(
    ctx: &Context,
    command: Command,
) -> Result<Report, CliError> {
    run_a_verb(ctx, command).map_err(|error| match error {
        CliError::UseCase(error) => {
            CliError::UseCase(ctx.dam().telling_whether_a_remote_holds(error))
        }
        error => error,
    })
}

fn run_a_verb(ctx: &Context, command: Command) -> Result<Report, CliError> {
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
        Command::Revert(a) => revert::run(ctx, a),
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
        Command::Sync(a) => sync::run_sync(ctx, a),
        Command::Category(a) => catalogue_write::run_category(ctx, a.command),
        Command::Filter(a) => catalogue_write::run_filter(ctx, a.command),
        Command::Hook(HookArgs {
            command: Some(HookCommand::Run(a)),
        }) => hook_run::run(ctx, a),
        Command::Init(_)
        | Command::Hook(_)
        | Command::Token(_)
        | Command::Config(_)
        | Command::Doctor(_)
        | Command::Test(_) => {
            unreachable!("answered before the store")
        }
    }
}

#[cfg(test)]
mod tests;
