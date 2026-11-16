use dam_application::RequiredField;
use dam_domain::Kind;

use crate::args::NewArgs;
use crate::error::CliError;

pub(crate) fn holds_each(args: &NewArgs, required: &[RequiredField]) -> Result<(), CliError> {
    if args.kind != Kind::Task {
        return Ok(());
    }
    match required.iter().find(|field| !held(args, **field)) {
        Some(field) => {
            let (needs, flag) = named(*field);
            Err(CliError::Usage(format!(
                "a task in this project needs {needs}; give it with {flag}"
            )))
        }
        None => Ok(()),
    }
}

fn held(args: &NewArgs, field: RequiredField) -> bool {
    match field {
        RequiredField::Due => args.due.is_some(),
        RequiredField::Labels => {
            !args.labels.is_empty() || args.fields.category.iter().any(|(_, l)| l != "none")
        }
        RequiredField::Priority => args.priority.is_some(),
        RequiredField::Body => !args.body.is_empty(),
        RequiredField::Deadline => args.deadline.is_some(),
    }
}

fn named(field: RequiredField) -> (&'static str, &'static str) {
    match field {
        RequiredField::Due => ("a due date", "--due"),
        RequiredField::Labels => ("a label", "--labels"),
        RequiredField::Priority => ("a priority", "--priority"),
        RequiredField::Body => ("a body", "--body"),
        RequiredField::Deadline => ("a deadline", "--deadline"),
    }
}
