use std::path::PathBuf;

use dam_domain::Permission::RemotePull;

use super::*;
use crate::config::HookEvent;
use crate::ports::{FunctionFailure, FunctionFault, HookFailure};
use crate::service::Dam;
use crate::testing::{allowed, holder};

fn no_bell() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("config.lua"),
        line: Some(2),
        fault: FunctionFault::Raised {
            message: "no bell".into(),
        },
    }
}

fn two_tasks() -> PullOutcome {
    PullOutcome {
        objects: vec![incoming("a", "r1"), incoming("b", "r2")],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    }
}

#[test]
fn the_first_failure_warns_once_however_many_tasks_fail() {
    let s = pulled_hooked();
    *s.fixture.hooks.failing_every_call.borrow_mut() = Some(no_bell());
    let report = pull(&s, two_tasks());
    assert_eq!(report.task_pulled.unwrap().failed, 2);
    assert_eq!(
        *s.fixture.hook_warnings.warned.borrow(),
        [HookFailure {
            event: HookEvent::TaskPulled,
            failure: no_bell(),
        }]
    );
}

#[test]
fn a_pull_a_tokens_read_caused_warns_nothing_and_reports_no_hook_run_to_the_token() {
    let s = pulled_hooked();
    *s.fixture.hooks.failing_every_call.borrow_mut() = Some(no_bell());
    let tool = holder(vec![allowed(&[RemotePull], None)]);
    let l = pulling(two_tasks());
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let report = dam.pull(remotes_launched_by(&l), None).unwrap().remove(0);
    assert_eq!(report.task_pulled, None);
    assert!(s.fixture.hook_warnings.warned.borrow().is_empty());
}
