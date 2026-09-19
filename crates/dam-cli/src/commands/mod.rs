mod done;
mod edit;
mod mv;
mod new;
mod rm;

use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub fn dispatch(ctx: &mut Context, command: Command) -> Result<Report, CliError> {
    match command {
        Command::New(a) => new::run(ctx, a),
        Command::Done(a) => done::run(ctx, a),
        Command::Edit(a) => edit::run(ctx, a),
        Command::Mv(a) => mv::run(ctx, a),
        Command::Rm(a) => rm::run(ctx, a),
        other => Err(CliError::Usage(format!("{other:?} is not implemented yet"))),
    }
}
