use dam_application::{ConfigError, ProjectRuleFault};
use dam_domain::Categories;

use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::project_scope::Standing;

pub(crate) fn following(
    mut ctx: Context,
    command: &Command,
    standing: &Standing,
) -> Result<Context, CliError> {
    let scope = match command {
        Command::New(a) => &a.scope,
        Command::Ls(a) => &a.scope,
        Command::Agenda(a) => &a.scope,
        Command::Reminders(a) => &a.scope,
        _ => return Ok(ctx),
    };
    let (Some(rules), Standing::Inside(project)) = (standing.rules(scope), standing) else {
        return Ok(ctx);
    };
    let mut both = ctx.config.categories.all().to_vec();
    both.extend(rules.categories.all().iter().cloned());
    ctx.config.categories = Categories::new(both).map_err(|_| {
        CliError::Config(ConfigError::ProjectRule {
            file: project.file(),
            key: "category",
            fault: ProjectRuleFault::ClashesWithYourCategories,
        })
    })?;
    Ok(ctx)
}
