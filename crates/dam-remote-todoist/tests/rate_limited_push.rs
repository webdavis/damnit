mod support;

use dam_application::HelperError;
use dam_application::UseCaseError;
use dam_application::testing::prelude::*;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::{FRESH_HELPER_PROCESSES_TRIED, the_real_helper_against};
use support::stateful_todoist::{serve, work_held_upstream};

const SUBJECTS: [&str; 3] = ["Ada", "tool", "work"];

#[test]
fn what_landed_before_a_rate_limit_is_kept_and_each_task_lands_once() {
    let _guard = testing::guard("what_landed_before_a_rate_limit_is_kept_and_each_task_lands_once")
        .allow_slow("each round launches the real helper process twice");
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        let s = Scenario::default();
        for (n, subject) in (1..).zip(SUBJECTS) {
            s.committed(Object::Task(task_in(n, subject, "Work/")));
        }
        let (server, upstream) = serve(work_held_upstream().rate_limiting_after(1));
        let dir = tempfile::tempdir().unwrap();
        let launcher = the_real_helper_against(&server.base, dir.path());

        let stop = s
            .dam()
            .push(remotes_launched_by(&launcher), None)
            .unwrap_err();
        let UseCaseError::PushStopped {
            by: HelperError::Remote(why),
            kept: 1,
            unanswered: 2,
        } = stop
        else {
            panic!("{stop:?}");
        };
        assert!(why.contains("retry after 42s"), "{why}");
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
}
