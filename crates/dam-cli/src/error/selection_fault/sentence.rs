use dam_domain::{DefaultsError, Has, Problem, select_field};

use super::Fault;
use crate::args::selection::flag_of;

pub(super) fn of_value(fault: &Fault, problem: &Problem) -> String {
    let (field, value) = (flag_of(fault.field()), fault.value());
    let typed = || {
        let texts = fault.typed.fields.get(fault.field());
        texts.map(|v| v.texts().join(" and ")).unwrap_or_default()
    };
    match problem {
        Problem::UnknownField { closest: Some(c) } => format!(
            "--any {field}={value} names no field; the closest is {}",
            flag_of(c)
        ),
        Problem::UnknownField { closest: None } => {
            format!(
                "--any {field}={value} names no field; {} --help shows the fields",
                fault.verb
            )
        }
        Problem::BadValue => format!(
            "--{field} cannot be {value:?}; it takes {}",
            fault.shapes().join(", ")
        ),
        Problem::TwoAtOnce { fix: Some(_) } => format!(
            "--{field} cannot be {} at once; a comma list holds when any of its values does",
            typed()
        ),
        Problem::TwoAtOnce { fix: None } => format!(
            "--{field} cannot be {} at once; --any holds when either does",
            typed()
        ),
        Problem::NotAGroup { groups } if groups.is_empty() => {
            format!(
                "--category {value}: no label group is declared; dam category create declares one"
            )
        }
        Problem::NotAGroup { groups } => format!(
            "--category {value}: that is not a label group; the groups are {}",
            groups.join(", ")
        ),
        Problem::NotInGroup { group, labels } => format!(
            "--category {value}: the group {group} holds only {}",
            labels.join(", ")
        ),
    }
}

pub(super) fn of_defaults(fault: &Fault, error: &DefaultsError) -> String {
    let lists = |field: &str| match select_field(field).map(|f| f.has) {
        Some(Has::Event) => "lists only events",
        Some(Has::Task) if field == "completed_at" => "lists only done tasks",
        _ => "lists only tasks",
    };
    let value = fault.value();
    match error {
        DefaultsError::KindContradicts { field } => format!(
            "--{} {}, and --kind {value} contradicts it",
            flag_of(field),
            lists(field)
        ),
        DefaultsError::DoneContradicts { field } => format!(
            "--{} {}, and --done {value} contradicts it",
            flag_of(field),
            lists(field)
        ),
        DefaultsError::BothKinds {
            task_field,
            event_field,
        } => format!(
            "--{} holds only for tasks and --{} only for events, so nothing has both; --any lists what has either",
            flag_of(task_field),
            flag_of(event_field)
        ),
        DefaultsError::EventFieldInHookRun { field } => format!(
            "--{} holds only for events, and a hook runs over tasks",
            flag_of(field)
        ),
    }
}

pub(super) fn of_sort(fault: &Fault, problem: &Problem) -> String {
    let value = fault.value();
    match problem {
        Problem::UnknownField { closest: Some(c) } => {
            format!("--sort {value} names no field; the closest is {c}")
        }
        Problem::UnknownField { closest: None } => {
            format!(
                "--sort {value} names no field; {} --help shows the fields",
                fault.verb
            )
        }
        _ => format!(
            "--sort cannot be {value:?}; it takes {}",
            fault.shapes().join(", ")
        ),
    }
}
