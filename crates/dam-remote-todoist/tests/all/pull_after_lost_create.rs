use crate::support;

use dam_application::ObjectRepository;
use dam_application::testing::{Scenario, remotes_launched_by, task_in};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::undocumented::EVERY_WAY_TODOIST_MIGHT_BEHAVE;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

fn named_ada_here(s: &Scenario) -> usize {
    let all = s.store.all().unwrap();
    all.iter().filter(|o| o.base().subject == "Ada").count()
}

#[test]
fn a_pull_after_a_create_whose_answer_was_lost_leaves_one_task_here_and_upstream() {
    let _guard = testing::guard(
        "a_pull_after_a_create_whose_answer_was_lost_leaves_one_task_here_and_upstream",
    )
    .allow_slow("launches the real helper process three times for each way");
    for how in EVERY_WAY_TODOIST_MIGHT_BEHAVE {
        let s = Scenario::default().drawing_from(42);
        s.committed(Object::Task(task_in(1, "Ada", "Work/")));
        let held = work_held_upstream()
            .behaving(how)
            .applying_the_next_write_then(Fault::HangUp);
        let (server, upstream) = serve(held);
        let dir = tempfile::tempdir().unwrap();
        let launcher = the_real_helper_against(&server.base, dir.path());
        let _ = s.dam().push(remotes_launched_by(&launcher), None);

        s.dam().pull(remotes_launched_by(&launcher), None).unwrap();
        let after_the_pull = named_ada_here(&s);
        let report = s
            .dam()
            .push(remotes_launched_by(&launcher), None)
            .unwrap()
            .remove(0);

        let upstream = upstream.lock().unwrap().items.len();
        let counts = (after_the_pull, named_ada_here(&s), upstream);
        assert_eq!(counts, (1, 1, 1), "{how:?}");
        assert!(report.failed.is_empty(), "{how:?}: {:?}", report.failed);
    }
}
