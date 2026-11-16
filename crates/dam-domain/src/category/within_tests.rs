use super::*;

fn category(name: &str, values: &[&str], exclusive: bool) -> Category {
    Category {
        name: name.into(),
        values: values.iter().map(|v| v.to_string()).collect(),
        exclusive,
    }
}

fn yours() -> Categories {
    Categories::new(vec![
        category("size", &["small", "big"], true),
        category("kind", &["bug", "chore"], false),
        category("place", &["home", "office"], false),
    ])
    .unwrap()
}

fn names(categories: &Categories) -> Vec<&str> {
    categories.all().iter().map(|c| c.name.as_str()).collect()
}

#[test]
fn a_project_category_with_your_name_sets_yours_aside_whole() {
    let project = Categories::new(vec![category("size", &["s", "m", "l"], true)]).unwrap();
    let within = yours().within(&project);
    assert_eq!(names(&within), ["size", "kind", "place"]);
    assert_eq!(
        within.category_of_name("size").unwrap().values,
        ["s", "m", "l"]
    );
    assert!(within.is_free("small"));
}

#[test]
fn a_project_category_sharing_one_label_sets_yours_aside_whole() {
    let project = Categories::new(vec![category("triage", &["bug", "feature"], true)]).unwrap();
    let within = yours().within(&project);
    assert_eq!(names(&within), ["triage", "size", "place"]);
    assert!(within.is_free("chore"));
    assert!(
        within
            .check(&["bug".into(), "feature".into()].into())
            .is_err()
    );
}

#[test]
fn set_aside_names_each_of_yours_with_the_project_category_taking_its_place() {
    let project = Categories::new(vec![
        category("size", &["s"], false),
        category("triage", &["bug"], false),
    ])
    .unwrap();
    assert_eq!(
        yours().set_aside_by(&project),
        [("size", "size"), ("kind", "triage")]
    );
}

#[test]
fn a_project_with_no_overlap_sets_nothing_aside() {
    let project = Categories::new(vec![category("phase", &["draft"], false)]).unwrap();
    assert!(yours().set_aside_by(&project).is_empty());
    assert_eq!(
        names(&yours().within(&project)),
        ["phase", "size", "kind", "place"]
    );
}
