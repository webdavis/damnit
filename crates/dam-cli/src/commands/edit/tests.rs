use super::*;
use crate::args::EditArgs;
use crate::testing::context;
use dam_domain::{Completion, Object, Oid, Task};

pub(super) fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

pub(super) fn args(oid: &Oid) -> EditArgs {
    EditArgs {
        oid: vec![oid.as_str().into()],
        set: Default::default(),
        editor: false,
        subject: None,
        body: None,
        priority: None,
        due: None,
        no_due: false,
        deadline: None,
        no_deadline: false,
        no_done: false,
        labels: vec![],
        unlabels: vec![],
        undepends: vec![],
        no_recurrence: false,
        no_attached: false,
        start: None,
        end: None,
        no_location: false,
        no_estimate: false,
        no_focus: false,
        fields: Default::default(),
        scope: Default::default(),
    }
}

#[test]
fn flags_change_fields() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    run(
        &ctx,
        EditArgs {
            subject: Some("oat milk".into()),
            due: Some("2026-09-25".into()),
            labels: vec!["errand".into()],
            ..args(&oid(1))
        },
    )
    .unwrap();
    let t = ctx.store.get(&oid(1)).unwrap().unwrap();
    assert_eq!(t.base().subject, "oat milk");
    assert!(t.base().labels.contains("errand"));
    assert_eq!(
        t.as_task()
            .unwrap()
            .due
            .as_ref()
            .map(|d| d.to_text())
            .as_deref(),
        Some("2026-09-25")
    );
}

#[test]
fn undone_reopens_a_completed_task_as_one_update_naming_done() {
    let ctx = context();
    let mut task = Task::new(oid(1), "milk");
    task.completion = Completion::Done { at: None };
    ctx.store.put(&Object::Task(task)).unwrap();
    ctx.dam()
        .add(&[dam_application::Named::Object(oid(1))])
        .unwrap();
    ctx.dam().commit("done").unwrap();
    run(
        &ctx,
        EditArgs {
            no_done: true,
            ..args(&oid(1))
        },
    )
    .unwrap();
    assert!(
        !ctx.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .completion
            .is_done()
    );
    let changes = ctx.dam().diff_working().unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].op(), dam_domain::Op::Update);
    assert_eq!(
        dam_domain::changed_fields(changes[0].before().unwrap(), changes[0].after().unwrap()),
        vec![dam_domain::Field::Done]
    );
}

#[test]
fn undone_on_an_open_task_is_refused_by_dams_own_rule() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let err = run(
        &ctx,
        EditArgs {
            no_done: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert!(err.to_string().contains(oid(1).short()), "{err}");
}

#[test]
fn a_date_is_set_by_today_with_a_count_and_a_period_is_refused() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let due = |text: &str| EditArgs {
        due: Some(text.into()),
        ..args(&oid(1))
    };
    run(&ctx, due("today+3")).unwrap();
    let t = ctx.store.get(&oid(1)).unwrap().unwrap();
    assert_eq!(
        t.as_task().unwrap().due,
        Some(dam_domain::When::Day(jiff::civil::date(2026, 9, 21)))
    );
    assert!(matches!(run(&ctx, due("2026-11")), Err(CliError::Usage(_))));
}
