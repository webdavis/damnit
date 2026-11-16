use dam_domain::{ChildDisposition, DependencyDisposition, Dispositions, Force, Tracked};

use super::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::edit::EditFields;

fn parent_of(s: &Scenario, oid: &Oid) -> Option<Oid> {
    s.store.get(oid).unwrap().unwrap().base().parent.clone()
}

fn two_errands() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"))
        .filed(task_in(2, "errands", "Home"));
    s.filed(task_in(3, "stamps", "Home/errands").held_by(1))
        .filed(task_in(4, "parcel", "Home/errands/later").held_by(1))
        .filed(task_in(6, "sweep", "Home/errands").held_by(2))
        .filed(task_in(7, "stairs", "Home/errands/sweep").held_by(6));
    s
}

trait HeldBy {
    fn held_by(self, byte: u8) -> Self;
}

impl HeldBy for dam_domain::Task {
    fn held_by(mut self, byte: u8) -> Self {
        self.base.parent = Some(oid(byte));
        self
    }
}

#[test]
fn removing_one_of_two_same_named_objects_moves_up_only_its_own_contents() {
    let s = two_errands();
    s.dam().remove(&oid(1)).unwrap();
    assert_eq!(s.path_of(&oid(3)), "Home/");
    assert_eq!(s.path_of(&oid(4)), "Home/later/");
    assert_eq!(parent_of(&s, &oid(3)), Some(oid(5)));
    assert_eq!(parent_of(&s, &oid(4)), Some(oid(5)));
    assert_eq!(s.path_of(&oid(6)), "Home/errands/");
    assert_eq!(s.path_of(&oid(7)), "Home/errands/sweep/");
}

#[test]
fn renaming_one_of_two_same_named_objects_carries_only_its_own_contents() {
    let s = two_errands();
    let rename = EditFields {
        subject: Some("chores".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(2), rename).unwrap();
    assert_eq!(s.path_of(&oid(6)), "Home/chores/");
    assert_eq!(s.path_of(&oid(7)), "Home/chores/sweep/");
    assert_eq!(parent_of(&s, &oid(6)), Some(oid(2)));
    assert_eq!(parent_of(&s, &oid(7)), Some(oid(6)));
    assert_eq!(s.path_of(&oid(3)), "Home/errands/");
    assert_eq!(s.path_of(&oid(4)), "Home/errands/later/");
}

#[test]
fn moving_one_of_two_same_named_objects_carries_only_its_own_contents() {
    let s = two_errands();
    s.dam()
        .relocate(&oid(1), &Path::parse("Work").unwrap())
        .unwrap();
    assert_eq!(s.path_of(&oid(3)), "Work/errands/");
    assert_eq!(s.path_of(&oid(4)), "Work/errands/later/");
    assert_eq!(s.path_of(&oid(6)), "Home/errands/");
}

#[test]
fn completing_one_of_two_same_named_objects_is_blocked_only_by_its_own_open_tasks() {
    let s = two_errands();
    let blockers = s.dam().plan_complete(&oid(2)).unwrap().blockers;
    assert_eq!(blockers.children, vec![oid(6)]);
}

#[test]
fn children_into_a_group_files_only_its_own_contents() {
    let s = two_errands();
    let into = Force::With(Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Keep,
    });
    s.dam().complete(&oid(2), into, Tracked::No).unwrap();
    assert_eq!(s.path_of(&oid(6)), "Home/leftovers/");
    assert_eq!(s.path_of(&oid(7)), "Home/leftovers/sweep/");
    assert_eq!(s.path_of(&oid(3)), "Home/errands/");
}

#[test]
fn moving_into_a_same_named_objects_contents_is_not_moving_inside_itself() {
    let s = two_errands();
    s.dam()
        .relocate(&oid(1), &Path::parse("Home/errands/sweep").unwrap())
        .unwrap();
    assert_eq!(parent_of(&s, &oid(1)), Some(oid(6)));
    assert_eq!(s.path_of(&oid(3)), "Home/errands/sweep/errands/");
}

#[test]
fn a_new_object_adopts_what_its_parent_holds_in_its_folder() {
    let s = Scenario::default().drawing_from(42);
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(3, "milk", "Home/errands/later"));
    let errands = crate::use_cases::new_object::NewTask {
        subject: "errands".into(),
        path: Path::parse("Home").unwrap(),
        priority: Default::default(),
        due: None,
        deadline: None,
        labels: Default::default(),
        body: String::new(),
        id: None,
    };
    let errands = s.dam().new_task(errands).unwrap().written();
    assert_eq!(parent_of(&s, &oid(3)), Some(errands.oid().clone()));
}

#[test]
fn a_renamed_object_adopts_only_what_its_parent_holds_in_its_new_folder() {
    let s = two_errands();
    s.filed(task_in(8, "chores", "Home"))
        .filed(task_in(9, "mop", "Home/chores"));
    s.store
        .put(&dam_domain::Object::Task(
            task_in(9, "mop", "Home/jobs").held_by(5),
        ))
        .unwrap();
    let rename = EditFields {
        subject: Some("errands".into()),
        ..EditFields::default()
    };
    let to_jobs = EditFields {
        subject: Some("jobs".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(8), to_jobs).unwrap();
    assert_eq!(parent_of(&s, &oid(9)), Some(oid(8)));
    s.dam().edit(&oid(8), rename).unwrap();
    assert_eq!(parent_of(&s, &oid(3)), Some(oid(1)));
    assert_eq!(parent_of(&s, &oid(6)), Some(oid(2)));
}

#[test]
fn a_command_on_contents_dam_cannot_place_is_refused_naming_the_namesakes() {
    let s = two_errands();
    s.store
        .put(&dam_domain::Object::Task(task_in(
            9,
            "bins",
            "Home/errands",
        )))
        .unwrap();
    let refused = crate::errors::Refusal::ParentNotKnown {
        oid: oid(1),
        unsettled: vec![oid(9)],
        namesakes: vec![oid(2)],
        folder: dam_domain::Path::parse("Home/errands").unwrap(),
        held_by_a_remote: false,
    };
    assert_eq!(s.dam().remove(&oid(1)), Err(refused.into()));
    assert_eq!(s.path_of(&oid(3)), "Home/errands/");
}
