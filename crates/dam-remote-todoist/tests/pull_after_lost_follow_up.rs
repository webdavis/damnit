mod support;

use dam_application::ObjectRepository;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::undocumented::EVERY_WAY_TODOIST_MIGHT_BEHAVE;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

fn ada(subject: &str) -> Object {
    Object::Task(task_in(1, subject, "Work/"))
}

fn tasks_named_ada_here(s: &Scenario) -> usize {
    let all = s.store.all().unwrap();
    all.iter()
        .filter(|o| o.base().subject.starts_with("Ada"))
        .count()
}

#[test]
fn a_rename_whose_follow_up_got_no_answer_reaches_todoist_after_a_pull() {
    let _guard =
        testing::guard("a_rename_whose_follow_up_got_no_answer_reaches_todoist_after_a_pull")
            .allow_slow("launches the real helper process four times for each way");
    for how in EVERY_WAY_TODOIST_MIGHT_BEHAVE {
        let s = Scenario::default().drawing_from(42);
        s.committed(ada("Ada"));
        let held = work_held_upstream()
            .behaving(how)
            .applying_the_next_write_then(Fault::HangUp);
        let (server, upstream) = serve(held);
        let dir = tempfile::tempdir().unwrap();
        let launcher = the_real_helper_against(&server.base, dir.path());
        let _ = s.dam().push(remotes_launched_by(&launcher), None);
        s.committed(ada("Ada and tool"));
        upstream.lock().unwrap().rate_limit_after(1);
        let _ = s.dam().push(remotes_launched_by(&launcher), None);

        s.dam().pull(remotes_launched_by(&launcher), None).unwrap();
        let here = s
            .store
            .get(&oid(1))
            .unwrap()
            .map(|o| o.base().subject.clone());
        let still_rate_limited = s.dam().push(remotes_launched_by(&launcher), None).is_err();
        if still_rate_limited {
            s.dam().push(remotes_launched_by(&launcher), None).unwrap();
        }

        assert_eq!(here.as_deref(), Some("Ada and tool"), "{how:?}");
        let upstream = upstream.lock().unwrap();
        assert_eq!(upstream.items.len(), 1, "{how:?}: {:?}", upstream.items);
        assert_eq!(upstream.item("Ada and tool").len(), 1, "{how:?}");
        assert_eq!(tasks_named_ada_here(&s), 1, "{how:?}");
    }
}
