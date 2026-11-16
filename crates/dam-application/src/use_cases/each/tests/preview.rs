use super::*;
use crate::config::{HookConfig, HookEvent};
use crate::testing::Said;

fn previewing() -> SetRun {
    SetRun {
        preview: true,
        ..run()
    }
}

#[test]
fn a_preview_answers_the_changes_and_writes_nothing() {
    let s = work_and_home();
    let before = s.store.clone();
    let out = s
        .dam()
        .edit_each(&named(&[1, 2]), retitled("x"), previewing())
        .unwrap();
    assert!(out.preview);
    assert_eq!(changed(&out), oids(&[1, 2]));
    assert_eq!(s.store, before);
}

#[test]
fn a_preview_runs_no_hook() {
    let s = work_and_home().configured(Config {
        hooks: vec![HookConfig {
            event: HookEvent::TaskChanged,
            on_failure: None,
            file: String::new(),
        }],
        ..config()
    });
    s.dam()
        .edit_each(&named(&[1]), retitled("x"), previewing())
        .unwrap();
    assert!(s.fixture.hooks.ran().is_empty());
}

#[test]
fn a_refused_preview_refuses_as_the_run_would() {
    let s = work_and_home();
    let fields = EditFields {
        add_depends: vec![oid(1)],
        ..EditFields::default()
    };
    assert!(
        s.dam()
            .edit_each(&named(&[1]), fields, previewing())
            .is_err()
    );
}

#[test]
fn a_run_reports_one_step_counting_the_objects() {
    let s = work_and_home();
    s.dam()
        .edit_each(&named(&[1, 2]), retitled("x"), run())
        .unwrap();
    let said = s.fixture.progress.said.borrow().clone();
    assert!(matches!(&said[0], Said::Started(_, Some(2))));
    assert_eq!(
        said[1..],
        [Said::Advanced(1), Said::Advanced(2), Said::Finished]
    );
}
