use super::*;

#[test]
fn every_rule_has_its_own_name() {
    let refusals = one_sample_of_every_refusal();
    let mut names: Vec<&str> = refusals.iter().map(Refusal::name).collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "two rules share a name: {names:?}");
}

#[test]
fn every_name_is_lower_snake_case_and_says_something() {
    for refusal in one_sample_of_every_refusal() {
        let name = refusal.name();
        assert!(!name.is_empty(), "{refusal:?}");
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{name} is not snake_case"
        );
    }
}
