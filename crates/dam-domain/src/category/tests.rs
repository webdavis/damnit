use super::*;

fn effort() -> Category {
    Category {
        name: "effort".into(),
        values: vec!["light".into(), "admin".into(), "deep".into()],
        exclusive: true,
    }
}

fn context() -> Category {
    Category {
        name: "context".into(),
        values: vec!["home".into(), "office".into()],
        exclusive: false,
    }
}

fn labels(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_value_in_two_categories_is_refused_at_definition() {
    let clash = Category {
        name: "mode".into(),
        values: vec!["deep".into()],
        exclusive: false,
    };
    let err = Categories::new(vec![effort(), clash]).unwrap_err();
    assert_eq!(
        err,
        CategoryError::DuplicateValue {
            value: "deep".into(),
            first: "effort".into(),
            second: "mode".into()
        }
    );
}

#[test]
fn an_empty_category_and_a_duplicate_name_are_refused() {
    let empty = Category {
        name: "x".into(),
        values: vec![],
        exclusive: true,
    };
    assert_eq!(
        Categories::new(vec![empty]).unwrap_err(),
        CategoryError::EmptyCategory("x".into())
    );
    assert_eq!(
        Categories::new(vec![effort(), effort()]).unwrap_err(),
        CategoryError::DuplicateName("effort".into())
    );
}

#[test]
fn one_value_from_an_exclusive_category_plus_free_labels_is_fine() {
    let cats = Categories::new(vec![effort(), context()]).unwrap();
    assert_eq!(cats.check(&labels(&["deep", "errand", "waiting"])), Ok(()));
}

#[test]
fn two_values_from_an_exclusive_category_are_refused() {
    let cats = Categories::new(vec![effort()]).unwrap();
    let err = cats
        .check(&labels(&["deep", "light", "errand"]))
        .unwrap_err();
    assert_eq!(
        err,
        LabelViolation::Exclusive {
            category: "effort".into(),
            held: vec!["deep".into(), "light".into()]
        }
    );
}

#[test]
fn several_values_from_a_multiple_category_are_fine() {
    let cats = Categories::new(vec![context()]).unwrap();
    assert_eq!(cats.check(&labels(&["home", "office"])), Ok(()));
}

#[test]
fn category_of_and_is_free() {
    let cats = Categories::new(vec![effort()]).unwrap();
    assert_eq!(
        cats.category_of("deep").map(|c| c.name.as_str()),
        Some("effort")
    );
    assert!(cats.is_free("errand"));
    assert!(!cats.is_free("deep"));
}

#[test]
fn category_of_name_finds_by_declared_name() {
    let cats = Categories::new(vec![effort()]).unwrap();
    assert!(cats.category_of_name("effort").is_some());
    assert!(cats.category_of_name("mood").is_none());
}

#[test]
fn no_categories_means_every_label_is_free() {
    let cats = Categories::new(vec![]).unwrap();
    assert_eq!(cats.check(&labels(&["a", "b"])), Ok(()));
}

#[test]
fn all_reports_every_declared_category_in_config_order() {
    let categories = Categories::new(vec![effort(), context()]).unwrap();
    let listed: Vec<&str> = categories.all().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(listed, ["effort", "context"]);
    assert_eq!(categories.all()[0].values, ["light", "admin", "deep"]);
    assert!(categories.all()[0].exclusive);
    assert!(!categories.all()[1].exclusive);
}

#[test]
fn all_is_empty_when_nothing_is_declared() {
    assert!(Categories::default().all().is_empty());
}
