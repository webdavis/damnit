use super::*;
use crate::testing::context;
use dam_application::Named;
use dam_domain::{Object, Task};

fn edited_after_a_commit(ctx: &Context) -> Oid {
    let id = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
    ctx.store
        .put(&Object::Task(Task::new(id.clone(), "milk")))
        .unwrap();
    ctx.dam().add(&[Named::Object(id.clone())]).unwrap();
    ctx.dam().commit("m").unwrap();
    let mut edited = ctx.store.get(&id).unwrap().unwrap();
    edited.base_mut().subject = "oat milk".into();
    ctx.store.put(&edited).unwrap();
    ctx.dam().add(&[Named::Object(id.clone())]).unwrap();
    id
}

#[test]
fn a_preview_shows_what_would_be_thrown_away_and_writes_nothing() {
    let ctx = context();
    let id = edited_after_a_commit(&ctx);
    let (objects, staged) = (ctx.store.all().unwrap(), ctx.store.staged().unwrap());
    let report = run(
        &ctx,
        RestoreArgs {
            oids: vec![id.to_string()],
            preview: true,
        },
    )
    .unwrap();
    assert_eq!(report.data["preview"], true);
    assert_eq!(report.data["changes"][0]["op"], "update");
    assert_eq!(report.data["changes"][0]["subject"], "milk");
    assert!(report.human.contains("nothing written"));
    assert_eq!(ctx.store.all().unwrap(), objects);
    assert_eq!(ctx.store.staged().unwrap(), staged);
}

#[test]
fn a_preview_refuses_as_the_run_would() {
    let ctx = context();
    let never = Oid::generate(&mut |x: &mut [u8]| x.fill(2));
    ctx.store
        .put(&Object::Task(Task::new(never.clone(), "x")))
        .unwrap();
    assert!(
        run(
            &ctx,
            RestoreArgs {
                oids: vec![never.to_string()],
                preview: true,
            },
        )
        .is_err()
    );
}
