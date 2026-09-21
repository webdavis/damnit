mod catalogue;
mod commit;
mod done;
mod edit;
mod ls;
mod mv;
mod new;
mod parsing;
mod remote;
mod restore;
mod rm;
mod show;
mod stage;
mod status;
mod sync;

use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

/// The verbs that pull a stale remote before they answer, which is the set
/// `--no-pull` acts on. Every one of them reaches `remote::maybe_pull_stale`.
pub(crate) fn pulls_a_stale_remote(command: &Command) -> bool {
    matches!(command, Command::Ls(_) | Command::Show(_))
}

pub(crate) fn dispatch(ctx: &mut Context, command: Command) -> Result<Report, CliError> {
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
        Command::Category(a) => catalogue::run_category(ctx, a.command),
        Command::Filter(a) => catalogue::run_filter(ctx, a.command),
        Command::Remote(a) => remote::run_remote(ctx, a),
        Command::Push(a) => sync::run_push(ctx, a),
        Command::Pull(a) => sync::run_pull(ctx, a),
        Command::Resolve(a) => sync::run_resolve(ctx, a),
    }
}
