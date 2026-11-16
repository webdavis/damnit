use super::fake::Fake;
use super::objects::{labelled, task};
use crate::select::{Condition, Problem};
use crate::{Categories, Category, Object};

fn fake() -> Fake {
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    fake.categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["deep".into(), "quick".into()],
        exclusive: true,
    }])
    .unwrap();
    fake
}

fn holds(value: &str, object: &Object) -> bool {
    let fake = fake();
    let condition = Condition::read("category", value).unwrap();
    condition.check(&fake.categories).unwrap();
    condition.holds(object, &fake)
}

#[test]
fn a_group_and_label_holds_that_label() {
    let deep = labelled(Object::Task(task(1)), &["deep"]);
    let quick = labelled(Object::Task(task(2)), &["quick"]);
    let bare = Object::Task(task(3));
    assert!(holds("effort=deep", &deep) && !holds("effort=deep", &quick));
    assert!(holds("effort=deep,quick", &quick));
    assert!(holds("effort", &quick) && !holds("effort", &bare));
    assert!(holds("not:effort", &bare) && !holds("not:effort", &deep));
    assert!(holds("not:effort=deep", &quick) && holds("not:effort=deep", &bare));
}

#[test]
fn a_label_outside_the_group_is_refused_listing_the_groups_labels() {
    let condition = Condition::read("category", "effort=home").unwrap();
    let error = condition.check(&fake().categories).unwrap_err();
    assert_eq!(
        error.problem,
        Problem::NotInGroup {
            group: "effort".into(),
            labels: vec!["deep".into(), "quick".into()]
        }
    );
}

#[test]
fn a_name_that_is_not_a_group_is_refused_listing_the_groups() {
    let condition = Condition::read("category", "mood=happy").unwrap();
    let error = condition.check(&fake().categories).unwrap_err();
    assert_eq!(
        error.problem,
        Problem::NotAGroup {
            groups: vec!["effort".into()]
        }
    );
    assert_eq!(error.value, "mood=happy");
}
