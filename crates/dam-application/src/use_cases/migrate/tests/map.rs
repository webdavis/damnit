use super::*;
use crate::use_cases::migrate::MapProblem;

fn problem(pairs: &[(&str, &str)]) -> MapProblem {
    LabelMap::new(
        pairs
            .iter()
            .map(|(old, new)| (old.to_string(), new.to_string()))
            .collect(),
        &effort(),
    )
    .unwrap_err()
}

#[test]
fn a_map_is_refused_when_it_cannot_say_one_new_label_for_each_old_one() {
    assert_eq!(problem(&[]), MapProblem::Empty);
    assert_eq!(
        problem(&[("quick", "light"), ("quick", "deep")]),
        MapProblem::Twice("quick".into())
    );
    assert_eq!(
        problem(&[("quick", "quick")]),
        MapProblem::Same("quick".into())
    );
    assert_eq!(problem(&[("quick", "")]), MapProblem::Blank);
    assert_eq!(problem(&[("", "light")]), MapProblem::Blank);
}

#[test]
fn a_new_label_another_category_holds_is_refused() {
    assert_eq!(
        problem(&[("quick", "home")]),
        MapProblem::TwoCategories {
            new: "home".into(),
            old: "quick".into(),
            old_category: "effort".into(),
            new_category: "place".into(),
        }
    );
}

#[test]
fn a_new_label_in_the_same_category_or_in_none_is_taken() {
    assert_eq!(map(&[("quick", "deep")]).pairs().len(), 1);
    assert_eq!(
        map(&[("quick", "light"), ("errand", "home")]).pairs().len(),
        2
    );
}
