use dam_domain::Path;

use crate::args::{Command, LabelCommand};
use crate::commands::catalogue_write;
use crate::commands::dispatch_a_verb_that_opens_the_store;
use crate::commands::mv::with_the_path_named;
use crate::commands::new::required::holds_each;
use crate::commands::new::templated::{filled, from_template};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

mod branch_label;
mod bulk;
mod categories;
mod project_created;

pub(crate) use categories::following;

pub(crate) fn dispatch_in(
    ctx: &Context,
    command: Command,
    standing: Standing,
) -> Result<Report, CliError> {
    crate::gate::admit(ctx.access.as_ref(), &command)?;
    let mut warnings = Vec::new();
    let command = match command {
        Command::New(a) => {
            let to_the_owner = ctx.access.token().is_none();
            let a = from_template(ctx.templates.as_ref(), a, to_the_owner)?;
            Command::New(match standing.rules(&a.scope) {
                Some(rules) => {
                    let (branch, warning) = branch_label::branch_of(&standing, rules);
                    warnings.extend(warning);
                    let mut a = filled(a, rules.new_task.clone());
                    a.labels.extend(branch.filter(|b| !a.labels.contains(b)));
                    holds_each(&a, &rules.required)?;
                    a
                }
                None => a,
            })
        }
        other => other,
    };
    let command = bulk::narrowing(command, &standing)?;
    let command = match command {
        Command::New(mut a) if a.parent.is_none() || a.path != Path::root() => {
            let place = standing.place(&a.scope)?;
            let created = match &a.scope.project {
                Some(project) if a.preview => {
                    a.hold = Some(project.clone());
                    None
                }
                Some(project) => ctx.dam().hold_project(project)?,
                None => None,
            };
            a.path = a.path.under(&place);
            let made = dispatch_a_verb_that_opens_the_store(ctx, Command::New(a))?;
            let mut told = project_created::told(made, created.as_ref(), &place);
            told.standard_error_lines.extend(warnings);
            return Ok(told);
        }
        Command::Mv(a) => {
            let mut a = with_the_path_named(a)?;
            if let Some(path) = a.path.take() {
                a.path = Some(path.under(&standing.place(&a.scope)?));
            }
            Command::Mv(a)
        }
        Command::List(mut a) => {
            if let Some(rules) = standing.rules(&a.scope) {
                if let Some(list) = rules.list.as_ref().filter(|_| a.selection.is_empty()) {
                    a.selection.typed = list.clone();
                }
                a.project_filters = rules.filters.clone();
            }
            a.place = standing.place(&a.scope)?;
            a.selection = a.selection.under(&a.place);
            Command::List(a)
        }
        Command::Agenda(mut a) => {
            if let Some(rules) = standing.rules(&a.scope) {
                a.project_filters = rules.filters.clone();
            }
            a.place = standing.place(&a.scope)?;
            a.selection = a.selection.under(&a.place);
            Command::Agenda(a)
        }
        Command::Done(mut a) => {
            a.project_children = standing.rules(&a.scope).and_then(|rules| {
                let place = Path::parse(&rules.path).ok()?;
                Some((place, rules.done_children.clone()?))
            });
            let mut answer = dispatch_a_verb_that_opens_the_store(ctx, Command::Done(a))?;
            answer.standard_error_lines.extend(standing.warning());
            return Ok(answer);
        }
        Command::Category(a) => return catalogue_write::run_category(ctx, a.command, &standing),
        Command::Filter(a) => return catalogue_write::run_filter(ctx, a.command, &standing),
        Command::Label(mut a) => {
            let LabelCommand::List(list) = &mut a.command;
            list.place = standing.place(&list.scope)?;
            Command::Label(a)
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
mod bulk_tests;
#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod mv_tests;
#[cfg(test)]
mod preview_tests;
#[cfg(test)]
mod project_created_tests;
#[cfg(test)]
mod rule_tests;
#[cfg(test)]
mod template_tests;
#[cfg(test)]
mod tests;
