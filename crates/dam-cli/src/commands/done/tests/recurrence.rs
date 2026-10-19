use super::super::run;
use super::{args, oid};
use crate::context::Context;
use crate::testing::context;
use dam_application::{RemoteConfig, RemoteName};
use dam_domain::{Object, Recurrence, Task};
fn a_task_with_a_rule_dam_cannot_read(ctx: &Context) {
    let mut t = Task::new(oid(7), "water the plants");
    t.base.recurrence = Some(Recurrence::parse("every other week"));
    ctx.store.put(&Object::Task(t)).unwrap();
}

#[test]
fn a_local_task_with_an_unreadable_rule_is_told_to_set_recurrence() {
    let ctx = context();
    a_task_with_a_rule_dam_cannot_read(&ctx);
    let err = run(&ctx, args(&oid(7), false, false)).unwrap_err();
    assert_eq!(
        err.to_string(),
        "cannot roll \"every other week\" forward; set --recurrence"
    );
}

#[test]
fn a_pulled_task_with_an_unreadable_rule_is_told_to_complete_it_upstream() {
    let mut ctx = context();
    a_task_with_a_rule_dam_cannot_read(&ctx);
    let todoist = RemoteName("todoist".into());
    ctx.config.remotes.push(RemoteConfig {
        name: todoist.clone(),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    });
    ctx.store.map_remote_id(&todoist, &oid(7), "4242").unwrap();
    let err = run(&ctx, args(&oid(7), false, false)).unwrap_err();
    assert_eq!(
        err.to_string(),
        "cannot roll \"every other week\" forward; complete it in Todoist or set --recurrence"
    );
}
