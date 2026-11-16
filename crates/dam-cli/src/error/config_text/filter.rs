use std::fmt;

use dam_application::{ConfigError, SelectError};
use dam_domain::{DefaultsError, FilterError, Problem};

pub(super) fn filter_text(f: &mut fmt::Formatter<'_>, error: &ConfigError) -> fmt::Result {
    match error {
        ConfigError::QueryText {
            file,
            was,
            key,
            table: Some(table),
        } => write!(
            f,
            "{was} in {file} is query text, which dam no longer reads; write {key} = {table} instead"
        ),
        ConfigError::QueryText { file, was, key, .. } => write!(
            f,
            "{was} in {file} is query text, which dam no longer reads; write {key} as a table of \
             dam list flags instead, such as {key} = {{ due = \"..today\", labels = \"bug\" }}"
        ),
        ConfigError::FilterTable { file, key, error } => match error.as_ref() {
            SelectError::Value(e) | SelectError::Sort(e) => value_text(f, file, key, e),
            SelectError::Defaults(e) => defaults_text(f, file, key, e),
        },
        ConfigError::FilterReachesItself { file, chain } => write!(
            f,
            "filter {} in {file} reaches itself ({}); remove one of those filter, in: or on: values",
            chain[0],
            chain.join(" -> ")
        ),
        _ => Ok(()),
    }
}

fn value_text(f: &mut fmt::Formatter<'_>, file: &str, key: &str, e: &FilterError) -> fmt::Result {
    let field = &e.field;
    write!(f, "{key}.{field} in {file} ")?;
    match &e.problem {
        Problem::UnknownField { closest: Some(c) } => {
            write!(f, "names no field; the closest is {c}")
        }
        Problem::UnknownField { closest: None } => {
            write!(f, "names no field; dam list --help shows the fields")
        }
        Problem::TwoAtOnce { fix: Some(fix) } => write!(
            f,
            "cannot hold two values at once; write {field} = \"{fix}\" for either"
        ),
        Problem::TwoAtOnce { fix: None } => write!(
            f,
            "cannot hold two values at once; put them under any for either"
        ),
        _ => write!(
            f,
            "cannot be {:?}; it takes {}",
            e.value,
            e.shapes().join(", ")
        ),
    }
}

fn defaults_text(
    f: &mut fmt::Formatter<'_>,
    file: &str,
    key: &str,
    e: &DefaultsError,
) -> fmt::Result {
    write!(f, "{key} in {file} ")?;
    match e {
        DefaultsError::BothKinds {
            task_field,
            event_field,
        } => write!(
            f,
            "names {task_field}, a task field, and {event_field}, an event field; put them under any for either"
        ),
        DefaultsError::KindContradicts { field } => {
            write!(f, "names {field}, which kind contradicts; drop kind")
        }
        DefaultsError::DoneContradicts { field } => {
            write!(f, "names {field}, which done contradicts; drop done")
        }
        DefaultsError::TaskFieldInAgenda { field }
        | DefaultsError::EventFieldInHookRun { field } => {
            write!(f, "names {field}, which this command refuses")
        }
    }
}
