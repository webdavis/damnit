mod support;

use std::time::Duration;

use dam_application::testing::prelude::*;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_application::{ConfiguredDuration, HelperError, UseCaseError};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::{serve, work_held_upstream};

const SUBJECTS: [&str; 3] = ["Ada", "tool", "work"];

#[test]
fn what_landed_before_the_deadline_is_kept_and_each_task_lands_once() {
    let _guard = testing::guard("what_landed_before_the_deadline_is_kept_and_each_task_lands_once")
        .allow_slow("the first push waits out the helper's silence and launches it twice");
    let mut s = Scenario::default();
    s.fixture.config.remotes[0].deadline = Some(ConfiguredDuration {
        value: Duration::from_secs(2),
        text: "2s".into(),
    });
    for (n, subject) in (1..).zip(SUBJECTS) {
        s.committed(Object::Task(task_in(n, subject, "Work/")));
    }
    let (held, release) = work_held_upstream().hanging_after(1);
    let (server, upstream) = serve(held);
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());

    let stop = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap_err();
    release.now();

    assert!(
        matches!(
            stop,
            UseCaseError::PushStopped {
                by: HelperError::Timeout { .. },
                kept: 1,
                unanswered: 2,
            }
        ),
        "{stop:?}"
    );
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
    assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 2);

    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let upstream = upstream.lock().unwrap();
    for subject in SUBJECTS {
        assert_eq!(upstream.item(subject).len(), 1, "{:?}", upstream.items);
    }
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}
