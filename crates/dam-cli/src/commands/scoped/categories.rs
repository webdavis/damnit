use dam_domain::Path;

use crate::args::Command;
use crate::context::Context;
use crate::project_scope::Standing;

pub(crate) fn following(mut ctx: Context, command: &Command, standing: &Standing) -> Context {
    let (scope, at_its_path) = match command {
        Command::New(a) => (&a.scope, false),
        Command::Ls(a) => (&a.scope, false),
        Command::Agenda(a) => (&a.scope, false),
        Command::Reminders(a) => (&a.scope, false),
        Command::Edit(a) => (&a.scope, true),
        Command::Done(a) => (&a.scope, true),
        Command::Mv(a) => (&a.scope, true),
        Command::Rm(a) => (&a.scope, true),
        _ => return ctx,
    };
    let Some(rules) = standing.rules(scope) else {
        return ctx;
    };
    let within = ctx.config.categories.within(&rules.categories);
    if !at_its_path {
        ctx.config.categories = within;
    } else if let Ok(place) = Path::parse(&rules.path) {
        ctx.config.project_categories = Some((place, within));
    }
    ctx
}
