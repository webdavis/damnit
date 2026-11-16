use crate::args::Command;
use crate::context::Context;
use crate::project_scope::Standing;

pub(crate) fn following(mut ctx: Context, command: &Command, standing: &Standing) -> Context {
    let scope = match command {
        Command::New(a) => &a.scope,
        Command::Ls(a) => &a.scope,
        Command::Agenda(a) => &a.scope,
        Command::Reminders(a) => &a.scope,
        _ => return ctx,
    };
    if let Some(rules) = standing.rules(scope) {
        ctx.config.categories = ctx.config.categories.within(&rules.categories);
    }
    ctx
}
