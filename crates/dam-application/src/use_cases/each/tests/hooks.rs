use super::*;
use crate::config::{HookConfig, HookEvent, OnFailure};
use crate::ports::{FunctionFailure, FunctionFault};

fn hooked(on_failure: Option<OnFailure>) -> Scenario {
    work_and_home().configured(Config {
        hooks: vec![HookConfig {
            event: HookEvent::TaskChanged,
            on_failure,
            file: String::new(),
        }],
        ..config()
    })
}

#[test]
fn hooks_run_once_per_changed_object() {
    let s = hooked(None);
    s.dam()
        .edit_each(&named(&[1, 2]), retitled("x"), run())
        .unwrap();
    assert_eq!(s.fixture.hooks.ran().len(), 2);
}

#[test]
fn a_refusing_hook_stops_the_whole_run_with_nothing_changed() {
    let s = hooked(Some(OnFailure::Refuse));
    *s.fixture.hooks.failing.borrow_mut() = Some(FunctionFailure {
        file: "config.lua".into(),
        line: None,
        fault: FunctionFault::Raised {
            message: "no".into(),
        },
    });
    let before = s.store.clone();
    assert!(
        s.dam()
            .edit_each(&named(&[1, 2]), retitled("x"), run())
            .is_err()
    );
    assert_eq!(s.store, before);
}
