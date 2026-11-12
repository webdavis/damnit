use crate::args::StatusArgs;
use crate::testing::{context, holding};
use dam_application::{Conflict, RemoteName};
use dam_domain::Permission::{ConfigRead, ConflictRead, StageRead, WorkingRead};
use dam_domain::{Object, Oid, Task};

fn a_conflict_and_a_warning() -> crate::context::Context {
    let mut ctx = context();
    let task = Object::Task(Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(1)), "t"));
    ctx.store
        .mark_conflict(&Conflict {
            remote: RemoteName("todoist".into()),
            oid: task.oid().clone(),
            ours: task.clone(),
            theirs: task,
        })
        .unwrap();
    ctx.config.tokens.warnings = vec!["token x: unknown permission \"a:b\" is skipped".into()];
    ctx
}

fn sections(ctx: &crate::context::Context) -> (usize, usize) {
    let report = super::run_status(ctx, StatusArgs { full: false }).unwrap();
    let count = |key: &str| report.data[key].as_array().unwrap().len();
    (count("conflicts"), count("warnings"))
}

#[test]
fn a_token_sees_conflicts_only_with_conflict_read_and_warnings_only_with_config_read() {
    let mut ctx = a_conflict_and_a_warning();
    ctx.access = holding(&[WorkingRead, StageRead]);
    assert_eq!(sections(&ctx), (0, 0));
    ctx.access = holding(&[WorkingRead, StageRead, ConflictRead, ConfigRead]);
    assert_eq!(sections(&ctx), (1, 1));
}

#[test]
fn the_owner_sees_every_section() {
    assert_eq!(sections(&a_conflict_and_a_warning()), (1, 1));
}
