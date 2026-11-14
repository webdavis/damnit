use crate::args::{AddArgs, ResetArgs};
use crate::testing::context;
use dam_domain::{Object, Oid, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

#[test]
fn add_stages_named_objects_and_all_stages_everything() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "a")))
        .unwrap();
    ctx.store
        .put(&Object::Task(Task::new(oid(2), "b")))
        .unwrap();
    let report = super::run_add(
        &ctx,
        AddArgs {
            oids: vec![oid(1).as_str().into()],
            all: false,
        },
    )
    .unwrap();
    assert_eq!(ctx.store.staged().unwrap().len(), 1);
    assert!(report.human.contains("a"));
    super::run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    assert_eq!(ctx.store.staged().unwrap().len(), 2);
}

#[test]
fn reset_unstages_one_or_all() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "a")))
        .unwrap();
    ctx.store
        .put(&Object::Task(Task::new(oid(2), "b")))
        .unwrap();
    super::run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    super::run_reset(
        &ctx,
        ResetArgs {
            oids: vec![oid(1).as_str().into()],
        },
    )
    .unwrap();
    assert_eq!(ctx.store.staged().unwrap().len(), 1);
    super::run_reset(&ctx, ResetArgs { oids: vec![] }).unwrap();
    assert!(ctx.store.staged().unwrap().is_empty());
}
