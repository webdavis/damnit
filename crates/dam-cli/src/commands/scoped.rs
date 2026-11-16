use dam_domain::Path;

use crate::args::Command;
use crate::commands::dispatch_a_verb_that_opens_the_store;
use crate::commands::mv::with_the_path_named;
use crate::commands::new::required::holds_each;
use crate::commands::new::templated::{filled, from_template};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

mod project_created;

pub(crate) fn dispatch_in(
    ctx: &Context,
    command: Command,
    standing: Standing,
) -> Result<Report, CliError> {
    crate::gate::admit(ctx.access.as_ref(), &command)?;
    let command = match command {
        Command::New(a) => {
            let to_the_owner = ctx.access.token().is_none();
            let a = from_template(ctx.templates.as_ref(), a, to_the_owner)?;
            Command::New(match standing.rules(&a.scope) {
                Some(rules) => {
                    let a = filled(a, rules.new_task.clone());
                    holds_each(&a, &rules.required)?;
                    a
                }
                None => a,
            })
        }
        other => other,
    };
    let command = match command {
        Command::New(mut a) if a.parent.is_none() || a.path != Path::root() => {
            let place = standing.place(&a.scope)?;
            let created = match &a.scope.project {
                Some(project) => ctx.dam().hold_project(project)?,
                None => None,
            };
            a.path = a.path.under(&place);
            let made = dispatch_a_verb_that_opens_the_store(ctx, Command::New(a))?;
            return Ok(project_created::told(made, created.as_ref(), &place));
        }
        Command::Mv(a) => {
            let mut a = with_the_path_named(a)?;
            if let Some(path) = a.path.take() {
                a.path = Some(path.under(&standing.place(&a.scope)?));
            }
            Command::Mv(a)
        }
        Command::Ls(mut a) => {
            if let Some(rules) = standing.rules(&a.scope) {
                a.query = rules.listed(a.query);
            }
            a.place = standing.place(&a.scope)?;
            Command::Ls(a)
        }
        Command::Agenda(mut a) => {
            a.place = standing.place(&a.scope)?;
            Command::Agenda(a)
        }
        Command::Reminders(mut a) => {
            a.place = standing.place(&a.scope)?;
            Command::Reminders(a)
        }
        other => {
            let mut answer = dispatch_a_verb_that_opens_the_store(ctx, other)?;
            answer.standard_error_lines.extend(standing.warning());
            return Ok(answer);
        }
    };
    dispatch_a_verb_that_opens_the_store(ctx, command)
}

#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod mv_tests;
#[cfg(test)]
mod project_created_tests;
#[cfg(test)]
mod rule_tests;
#[cfg(test)]
mod template_tests;
#[cfg(test)]
mod tests;
