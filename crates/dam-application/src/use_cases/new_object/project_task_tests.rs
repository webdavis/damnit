use dam_domain::Permission::{TaskCreate, TaskRead};
use dam_domain::{Object, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};

fn place(text: &str) -> Path {
    Path::parse(text).unwrap()
}

fn written_subject(landed: Option<crate::use_cases::items::land::Landed>) -> Option<String> {
    landed.map(|l| l.written().base().subject.clone())
}

#[test]
fn a_missing_project_task_is_created_under_its_parent_path() {
    let s = Scenario::default();
    let made = s.dam().hold_project(&place("pns/hooks/")).unwrap();
    assert_eq!(written_subject(made), Some("hooks".into()));
    let all = s.dam().list_within(&dam_domain::Path::root()).unwrap();
    assert_eq!(all[0].base().path, place("pns/"));
}

#[test]
fn a_project_task_already_there_is_kept_and_the_root_needs_none() {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "hooks", "pns")));
    assert!(
        s.dam()
            .hold_project(&place("pns/hooks/"))
            .unwrap()
            .is_none()
    );
    assert!(s.dam().hold_project(&Path::root()).unwrap().is_none());
    assert_eq!(
        s.dam()
            .list_within(&dam_domain::Path::root())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_token_without_create_at_the_path_is_refused_before_anything_is_looked_up() {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "hooks", "pns")));
    let refused = |grants| {
        let token = holder(grants);
        let dam = Dam {
            access: &token,
            ..s.dam()
        };
        dam.hold_project(&place("pns/hooks/")).unwrap_err()
    };
    let nothing = refused(vec![allowed(&[TaskRead], None)]);
    let elsewhere = refused(vec![allowed(&[TaskCreate, TaskRead], Some(&["home"]))]);
    for refusal in [nothing, elsewhere] {
        assert!(matches!(
            refusal,
            UseCaseError::Refused(Refusal::NotPermitted(TaskCreate))
        ));
    }
}

#[test]
fn a_token_that_cannot_see_the_project_task_learns_nothing_of_it() {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "hooks", "pns")));
    let token = holder(vec![allowed(&[TaskCreate], Some(&["pns"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert!(dam.hold_project(&place("pns/hooks/")).unwrap().is_some());
}
