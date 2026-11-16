use super::super::*;
use crate::config::Config;
use crate::errors::Refusal;
use crate::testing::{Scenario, oid, task};
use dam_domain::{Categories, Category};

#[test]
fn labels_are_added_removed_and_checked() {
    let cats = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["light".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap();
    let s = Scenario::default().configured(Config {
        categories: cats,
        ..Config::default()
    });
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    dam.edit(
        &id,
        EditFields {
            add_labels: vec!["deep".into(), "errand".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let err = dam
        .edit(
            &id,
            EditFields {
                add_labels: vec!["light".into()],
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
    let out = dam
        .edit(
            &id,
            EditFields {
                remove_labels: vec!["deep".into()],
                add_labels: vec!["light".into()],
                ..Default::default()
            },
        )
        .unwrap()
        .written();
    assert!(out.base().labels.contains("light") && !out.base().labels.contains("deep"));
}

#[test]
fn the_categories_of_a_place_hold_only_for_objects_under_it() {
    let place = dam_domain::Path::parse("work/").unwrap();
    let s = Scenario::default().configured(Config {
        project_categories: Some((
            place,
            Categories::new(vec![Category {
                name: "stage".into(),
                values: vec!["review".into(), "ship".into()],
                exclusive: true,
            }])
            .unwrap(),
        )),
        ..Config::default()
    });
    s.working(Object::Task(crate::testing::task_in(1, "in", "work")));
    s.working(Object::Task(crate::testing::task_in(2, "out", "home")));
    let both = || EditFields {
        add_labels: vec!["review".into(), "ship".into()],
        ..Default::default()
    };
    let refused = s.dam().edit(&oid(1), both()).unwrap_err();
    assert!(matches!(refused, UseCaseError::Refused(Refusal::Labels(_))));
    assert!(s.dam().edit(&oid(2), both()).is_ok());
}
