use crate::args::{Command, ScopeArgs, SetArgs};
use crate::error::CliError;
use crate::project_scope::Standing;

pub(super) fn narrowing(command: Command, standing: &Standing) -> Result<Command, CliError> {
    Ok(match command {
        Command::Edit(mut a) => {
            narrowed(&mut a.set, &a.scope, standing)?;
            Command::Edit(a)
        }
        Command::Done(mut a) => {
            narrowed(&mut a.set, &a.scope, standing)?;
            Command::Done(a)
        }
        Command::Mv(mut a) => {
            narrowed(&mut a.set, &a.scope, standing)?;
            Command::Mv(a)
        }
        Command::Rm(mut a) => {
            narrowed(&mut a.set, &a.scope, standing)?;
            Command::Rm(a)
        }
        other => other,
    })
}

fn narrowed(set: &mut SetArgs, scope: &ScopeArgs, standing: &Standing) -> Result<(), CliError> {
    if set.filter.is_none() {
        return Ok(());
    }
    set.place = standing.place(scope)?;
    if let Some(rules) = standing.rules(scope) {
        set.project_filters = rules.filters.clone();
    }
    Ok(())
}
