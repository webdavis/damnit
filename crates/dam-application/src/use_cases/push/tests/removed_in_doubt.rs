use dam_domain::{Object, Path};

use crate::ports::{DoubtRepository, Notice, NoticeRepository};
use crate::testing::{Scenario, oid, pushing, remotes_launched_by, task_in, todoist};

fn removed_in_doubt() -> Notice {
    Notice::CreateInDoubtRemoved {
        remote: todoist(),
        oid: oid(1),
        subject: "Ada".into(),
        path: Path::parse("Work/").unwrap(),
    }
}

fn notices_of_the_removal(s: &Scenario) -> usize {
    let notices = s.store.notices().unwrap();
    notices.iter().filter(|n| **n == removed_in_doubt()).count()
}

#[test]
fn a_create_in_doubt_removed_before_its_resend_leaves_one_notice_naming_it() {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.deleted(&oid(1));

    let (l, sent) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(notices_of_the_removal(&s), 1);
    assert!(sent.borrow().is_empty(), "{:?}", sent.borrow());
    assert!(
        !s.store
            .fields_in_doubt(&todoist())
            .unwrap()
            .contains_key(&oid(1))
    );

    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(notices_of_the_removal(&s), 1, "the notice is left once");
}

#[test]
fn a_removed_object_whose_create_was_answered_leaves_no_such_notice() {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    let (l, _) = pushing(crate::testing::refusing_all("no"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.deleted(&oid(1));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(notices_of_the_removal(&s), 0);
}
