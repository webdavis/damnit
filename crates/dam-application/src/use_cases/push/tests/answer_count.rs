use dam_domain::{Object, Task};

use crate::ports::AnswerRepository;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task, todoist,
};

fn answers_about_1(s: &Scenario) -> u64 {
    s.store.answers(&todoist(), &oid(1)).unwrap()
}

#[test]
fn the_count_moves_on_accepted_and_refused_and_not_on_unanswered() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(answers_about_1(&s), 1, "accepted");

    s.committed(Object::Task(Task::new(oid(1), "second")));
    let (l, _) = pushing(refusing_all("temporary failure"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(answers_about_1(&s), 2, "refused");

    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(answers_about_1(&s), 2, "unanswered");
}
