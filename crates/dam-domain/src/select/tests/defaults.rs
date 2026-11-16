use super::tables::{either, one, table};
use crate::select::{Command, DefaultsError, Filter, FilterTable, Fix, Scope, scope};

fn of(t: &FilterTable, command: Command) -> Result<Scope, DefaultsError> {
    scope(&Filter::read(t).unwrap(), command)
}

fn list(fields: &[(&str, &str)]) -> Result<Scope, DefaultsError> {
    let fields: Vec<_> = fields.iter().map(|(k, v)| (*k, one(v))).collect();
    of(&table(&fields), Command::List)
}

fn agenda(fields: &[(&str, &str)]) -> Result<Scope, DefaultsError> {
    let fields: Vec<_> = fields.iter().map(|(k, v)| (*k, one(v))).collect();
    of(&table(&fields), Command::Agenda)
}

const OPEN_TASKS: Scope = Scope {
    tasks: true,
    events: false,
    done: Some(false),
};
const EVENTS: Scope = Scope {
    tasks: false,
    events: true,
    done: None,
};

#[test]
fn list_lists_open_tasks_while_other_fields_are_typed() {
    assert_eq!(list(&[]), Ok(OPEN_TASKS));
    assert_eq!(
        list(&[("labels", "bug"), ("due", "..today")]),
        Ok(OPEN_TASKS)
    );
}

#[test]
fn any_lifts_a_default() {
    let all = Scope {
        tasks: true,
        events: true,
        done: None,
    };
    assert_eq!(list(&[("kind", "any"), ("done", "any")]), Ok(all));
    assert_eq!(
        list(&[("done", "true")]),
        Ok(Scope {
            done: Some(true),
            ..OPEN_TASKS
        })
    );
    assert_eq!(
        list(&[("kind", "event")]).map(|s| (s.tasks, s.events)),
        Ok((false, true))
    );
}

#[test]
fn an_event_field_means_events_and_completed_at_means_done_tasks() {
    assert_eq!(
        list(&[("location", "not:home")]).map(|s| (s.tasks, s.events)),
        Ok((false, true))
    );
    assert_eq!(
        list(&[("completed_at", "week")]),
        Ok(Scope {
            done: Some(true),
            ..OPEN_TASKS
        })
    );
    assert_eq!(list(&[("completed_at", "none")]), Ok(OPEN_TASKS));
}

#[test]
fn a_typed_kind_or_done_that_contradicts_a_field_exits_with_the_flag_to_drop() {
    let kind = list(&[("kind", "task"), ("start", "today")]).unwrap_err();
    assert_eq!(kind, DefaultsError::KindContradicts { field: "start" });
    assert_eq!(kind.fix(), Fix::Drop("kind"));
    assert!(list(&[("done", "any"), ("completed_at", "today")]).is_ok());
    let done = list(&[("done", "false"), ("completed_at", "today")]).unwrap_err();
    assert_eq!(
        done,
        DefaultsError::DoneContradicts {
            field: "completed_at"
        }
    );
    assert_eq!(done.fix(), Fix::Drop("done"));
    let event_done = list(&[("done", "true"), ("start", "today")]).unwrap_err();
    assert_eq!(
        event_done,
        DefaultsError::DoneContradicts { field: "start" }
    );
    assert_eq!(
        list(&[("kind", "event"), ("start", "today")]).map(|s| s.events),
        Ok(true)
    );
}

#[test]
fn a_task_field_and_an_event_field_together_point_at_any() {
    let both = list(&[("due", "today"), ("start", "today")]).unwrap_err();
    assert_eq!(
        both,
        DefaultsError::BothKinds {
            task_field: "due",
            event_field: "start"
        }
    );
    assert_eq!(both.fix(), Fix::UseAny);
}

#[test]
fn a_field_inside_any_never_sets_kind_unless_any_names_both_kinds() {
    let alternatives = |a: (&str, &str), b: (&str, &str)| {
        either(
            table(&[]),
            vec![table(&[(a.0, one(a.1))]), table(&[(b.0, one(b.1))])],
        )
    };
    let events_only = alternatives(("start", "today"), ("location", "x"));
    assert_eq!(of(&events_only, Command::List), Ok(OPEN_TASKS));
    let both = alternatives(("due", "today"), ("start", "today"));
    assert_eq!(
        of(&both, Command::List).map(|s| (s.tasks, s.events)),
        Ok((true, true))
    );
    let completed = alternatives(("completed_at", "today"), ("labels", "x"));
    assert_eq!(of(&completed, Command::List), Ok(OPEN_TASKS));
}

#[test]
fn one_any_alone_is_the_same_as_the_plain_field() {
    let alone = either(table(&[]), vec![table(&[("start", one("today"))])]);
    assert_eq!(
        of(&alone, Command::List).map(|s| (s.tasks, s.events)),
        Ok((false, true))
    );
}

#[test]
fn agenda_lists_events_and_sends_task_fields_to_list() {
    assert_eq!(agenda(&[]), Ok(EVENTS));
    assert_eq!(agenda(&[("location", "x")]), Ok(EVENTS));
    let task_field = agenda(&[("due", "today")]).unwrap_err();
    assert_eq!(
        task_field,
        DefaultsError::TaskFieldInAgenda { field: "due" }
    );
    assert_eq!(task_field.fix(), Fix::UseList);
    let kind = agenda(&[("kind", "task")]).unwrap_err();
    assert_eq!(kind, DefaultsError::TaskFieldInAgenda { field: "kind" });
    assert_eq!(agenda(&[("kind", "any")]).map(|s| s.tasks), Ok(true));
}
