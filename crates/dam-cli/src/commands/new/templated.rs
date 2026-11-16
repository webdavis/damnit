use dam_application::{TaskTemplate, TaskTemplates, TemplateName, TemplateOwner};
use dam_domain::Path;

use crate::args::NewArgs;
use crate::error::CliError;

const NOT_A_NAME: &str = "--template takes a template's file name without .lua, such as bug";

pub(crate) fn from_template(
    templates: &dyn TaskTemplates,
    args: NewArgs,
    names_are_shown: bool,
) -> Result<NewArgs, CliError> {
    let Some(asked) = args.template.as_deref() else {
        return Ok(args);
    };
    let name = TemplateName::parse(asked).map_err(|_| CliError::Usage(NOT_A_NAME.into()))?;
    let Some(template) = templates.find(&name)? else {
        let text = match names_are_shown {
            true => not_found(&name, &templates.names()?),
            false => format!("no template named {}", name.as_str()),
        };
        return Err(CliError::Usage(text));
    };
    Ok(filled(args, template))
}

pub(crate) fn filled(args: NewArgs, template: TaskTemplate) -> NewArgs {
    NewArgs {
        body: match args.body.is_empty() {
            true => template.body.unwrap_or_default(),
            false => args.body,
        },
        labels: match args.labels.is_empty() {
            true => template.labels.unwrap_or_default(),
            false => args.labels,
        },
        path: match (args.path == Path::root(), template.path) {
            (true, Some(path)) => path,
            _ => args.path,
        },
        priority: args.priority.or(template.priority),
        due: args.due.or(template.due),
        deadline: args.deadline.or(template.deadline),
        ..args
    }
}

fn not_found(name: &TemplateName, names: &[(String, TemplateOwner)]) -> String {
    let name = name.as_str();
    if names.is_empty() {
        return format!(
            "no template named {name}, and there are none; add templates/{name}.lua beside \
             config.lua"
        );
    }
    let listed: Vec<String> = names
        .iter()
        .map(|(held, owner)| match owner {
            TemplateOwner::Project => format!("{held} (the project's)"),
            TemplateOwner::Person => format!("{held} (yours)"),
        })
        .collect();
    let listed = match listed.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    };
    format!("no template named {name}; the templates are {listed}")
}
