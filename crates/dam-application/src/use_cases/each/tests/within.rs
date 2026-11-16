use dam_domain::Permission::{TaskChange, TaskRead};
use dam_domain::{Categories, Category, Path};

use super::*;
use crate::service::Dam;
use crate::testing::{allowed, holder, task_in};

fn work() -> Path {
    Path::parse("work/").unwrap()
}

fn in_work_and_beside_it() -> Scenario {
    let s = work_and_home();
    s.working(Object::Task(task_in(4, "client", "work/clients")));
    s.working(Object::Task(task_in(5, "other clients", "clients")));
    s
}

fn sorted(out: &SetChanges) -> Vec<Oid> {
    let mut oids = out.oids.clone();
    oids.sort();
    oids
}

#[test]
fn a_query_within_a_place_selects_only_what_lies_under_it() {
    let s = in_work_and_beside_it();
    let within = Selector::Query("kind:task".into(), work());
    let out = s.dam().edit_each(&within, retitled("x"), run()).unwrap();
    assert_eq!(sorted(&out), oids(&[1, 2, 4]));
}

#[test]
fn a_path_in_a_query_within_a_place_starts_from_that_place() {
    let s = in_work_and_beside_it();
    let within = Selector::Query("path:clients/".into(), work());
    let out = s.dam().edit_each(&within, retitled("x"), run()).unwrap();
    assert_eq!(out.oids, oids(&[4]));
}

#[test]
fn a_filter_within_a_place_selects_only_what_lies_under_it() {
    let s = in_work_and_beside_it();
    let within = Selector::Filter("home".into(), work());
    let out = s.dam().edit_each(&within, retitled("x"), run()).unwrap();
    assert!(out.oids.is_empty());
}

#[test]
fn a_query_within_a_place_reads_the_categories_of_that_place() {
    let place_categories = Categories::new(vec![Category {
        name: "stage".into(),
        values: vec!["review".into()],
        exclusive: true,
    }])
    .unwrap();
    let s = Scenario::default().configured(Config {
        project_categories: Some((work(), place_categories)),
        ..config()
    });
    s.working(Object::Task(task_in(1, "report", "work")));
    let within = Selector::Query("stage:review".into(), work());
    assert!(s.dam().edit_each(&within, retitled("x"), run()).is_ok());
    let everywhere = Selector::Query("stage:review".into(), Path::root());
    assert!(
        s.dam()
            .edit_each(&everywhere, retitled("x"), run())
            .is_err()
    );
}

#[test]
fn a_token_within_a_place_selects_only_what_it_sees_there() {
    let s = in_work_and_beside_it();
    let token = holder(vec![allowed(
        &[TaskRead, TaskChange],
        Some(&["work/clients", "clients"]),
    )]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let within = Selector::Query("kind:task".into(), work());
    let out = dam.edit_each(&within, retitled("x"), run()).unwrap();
    assert_eq!(out.oids, oids(&[4]));
}
