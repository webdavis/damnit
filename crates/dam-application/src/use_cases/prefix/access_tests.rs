use dam_domain::Object;
use dam_domain::Permission::TaskRead;

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, oid, task_in};

#[test]
fn a_prefix_shared_with_an_invisible_object_resolves_to_the_visible_one() {
    let s = Scenario::default();
    s.working(Object::Task(task_in(0xab, "seen", "work")));
    let mut hidden = task_in(0xac, "hidden", "home");
    hidden.base.oid = dam_domain::Oid::parse(&format!("abab{}", &oid(0xac).as_str()[4..])).unwrap();
    s.working(Object::Task(hidden));
    let token = holder(vec![allowed(&[TaskRead], Some(&["work"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(dam.resolve_object("abab").unwrap(), oid(0xab));
    assert!(
        s.dam().resolve_object("abab").is_err(),
        "the owner sees both"
    );
}

#[test]
fn a_prefix_matching_only_invisible_objects_matches_nothing() {
    let s = Scenario::default();
    s.working(Object::Task(task_in(0xab, "hidden", "home")));
    let token = holder(vec![allowed(&[TaskRead], Some(&["work"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        dam.resolve_object("abab"),
        Err(UseCaseError::Refused(Refusal::NoWorkingObject(
            "abab".into()
        )))
    );
}
