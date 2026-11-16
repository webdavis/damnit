use super::tables::{one, table};
use crate::select::{Command, DefaultsError, Filter, Fix, Scope, scope};

fn hook(fields: &[(&str, &str)]) -> Result<Scope, DefaultsError> {
    let fields: Vec<_> = fields.iter().map(|(k, v)| (*k, one(v))).collect();
    scope(&Filter::read(&table(&fields)).unwrap(), Command::HookRun)
}

const EVERY_TASK: Scope = Scope {
    tasks: true,
    events: false,
    done: None,
};

#[test]
fn hook_run_runs_over_every_task_done_or_not_unless_done_is_typed() {
    assert_eq!(hook(&[]), Ok(EVERY_TASK));
    assert_eq!(hook(&[("labels", "bug")]), Ok(EVERY_TASK));
    assert_eq!(hook(&[("kind", "any")]), Ok(EVERY_TASK));
    assert_eq!(hook(&[("done", "false")]).map(|s| s.done), Ok(Some(false)));
}

#[test]
fn an_event_field_in_hook_run_points_at_list() {
    let field = hook(&[("start", "today")]).unwrap_err();
    assert_eq!(field, DefaultsError::EventFieldInHookRun { field: "start" });
    assert_eq!(field.fix(), Fix::UseList);
    let kind = hook(&[("kind", "event")]).unwrap_err();
    assert_eq!(kind, DefaultsError::EventFieldInHookRun { field: "kind" });
}
