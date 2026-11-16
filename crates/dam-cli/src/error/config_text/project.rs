use std::fmt;

use dam_application::{ConfigError, ProjectRuleFault};

pub(super) fn project_text(f: &mut fmt::Formatter<'_>, error: &ConfigError) -> fmt::Result {
    match error {
        ConfigError::BelongsInPersonalConfig { key, file } => write!(
            f,
            "{key} belongs in your own config.lua, never in a project file; remove it from {}",
            file.display()
        ),
        ConfigError::HoldsAControlCharacter { file } => write!(
            f,
            "{} holds a control character in a key or a text; remove it",
            file.display()
        ),
        ConfigError::NotAPath { file } => write!(
            f,
            "path in {} is not a path such as work/app/; correct it",
            file.display()
        ),
        ConfigError::NeedsANewerDam { file, needs, this } => write!(
            f,
            "{} needs dam {needs} or newer; this is dam {this}",
            file.display()
        ),
        ConfigError::ProjectRule { file, key, fault } => rule_text(f, &file.display(), key, *fault),
        _ => Ok(()),
    }
}

fn rule_text(
    f: &mut fmt::Formatter<'_>,
    file: &dyn fmt::Display,
    key: &str,
    fault: ProjectRuleFault,
) -> fmt::Result {
    match fault {
        ProjectRuleFault::Shape => write!(
            f,
            "{key} in {file} is not written as dam reads it; correct it, then run dam doctor config"
        ),
        ProjectRuleFault::FilterIsAFunction => write!(
            f,
            "a filter in {file} holds a function; a project's filter is a query"
        ),
        ProjectRuleFault::LabelInTwoCategories => write!(
            f,
            "a label in {file} belongs to two of its categories; keep each label in one category"
        ),
        ProjectRuleFault::NotAQuery => write!(
            f,
            "{key} in {file} is not a query dam can read; correct it, then run dam doctor config"
        ),
    }
}
