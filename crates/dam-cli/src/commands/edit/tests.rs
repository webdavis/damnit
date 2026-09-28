use super::*;
use crate::args::EditArgs;
use crate::testing::{ScriptedEditor, context, machine_context};
use dam_domain::{Object, Oid, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn args(oid: &Oid) -> EditArgs {
    EditArgs {
        oid: oid.as_str().into(),
        editor: false,
        subject: None,
        body: None,
        priority: None,
        due: None,
        no_due: false,
        deadline: None,
        no_deadline: false,
        undone: false,
        labels: vec![],
        unlabels: vec![],
        depends: vec![],
        undepends: vec![],
        recurrence: None,
        no_recurrence: false,
        attach: None,
        detach: false,
        start: None,
        end: None,
        location: None,
        no_location: false,
    }
}

#[test]
fn flags_change_fields() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    run(
        &mut ctx,
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
fn the_editor_round_trip_applies_the_saved_text() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let saved = dam_adapters::render_template(&ctx.store.get(&oid(1)).unwrap().unwrap())
        .replace("subject = \"milk\"", "subject = \"eggs\"");
    ctx.editor = Some(Box::new(ScriptedEditor(saved)));
    run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "eggs"
    );
}

#[test]
fn an_unchanged_editor_save_is_a_cancel() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let same = dam_adapters::render_template(&ctx.store.get(&oid(1)).unwrap().unwrap());
    ctx.editor = Some(Box::new(ScriptedEditor(same)));
    assert!(matches!(
        run(
            &mut ctx,
            EditArgs {
                editor: true,
                ..args(&oid(1))
            }
        ),
        Err(CliError::Cancelled)
    ));
}

#[test]
fn a_save_that_never_parses_stops_after_it_comes_back_unchanged() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor("priority = \"high\"\n".into())));
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("priority"), "{err}");
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "milk"
    );
}

#[test]
fn a_machine_format_refuses_to_open_an_editor() {
    let mut ctx = machine_context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.to_string(), Refusal::NeedsAnEditor.to_string());
    assert_eq!(err.exit_code(), 4);
}

#[test]
fn undone_reopens_a_completed_task_as_one_update_naming_done() {
    let mut ctx = context();
    let mut task = Task::new(oid(1), "milk");
    task.done = true;
    ctx.store.put(&Object::Task(task)).unwrap();
    dam_application::add(ctx.store.as_ref(), ctx.store.as_ref(), &[oid(1)]).unwrap();
    dam_application::commit(
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        "done",
    )
    .unwrap();
    run(
        &mut ctx,
        EditArgs {
            undone: true,
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
            .done
    );
    let changes =
        dam_application::diff_working(ctx.store.as_ref(), ctx.store.as_ref(), ctx.store.as_ref())
            .unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].op, dam_domain::Op::Update);
    assert_eq!(
        dam_domain::changed_fields(
            changes[0].before.as_ref().unwrap(),
            changes[0].after.as_ref().unwrap()
        ),
        vec![dam_domain::Field::Done]
    );
}

#[test]
fn undone_on_an_open_task_is_refused_by_dams_own_rule() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let err = run(
        &mut ctx,
        EditArgs {
            undone: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert!(err.to_string().contains(oid(1).short()), "{err}");
}
