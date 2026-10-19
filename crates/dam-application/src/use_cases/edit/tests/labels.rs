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
        .unwrap();
    assert!(out.base().labels.contains("light") && !out.base().labels.contains("deep"));
}
