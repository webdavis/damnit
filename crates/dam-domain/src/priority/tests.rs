use super::*;

#[test]
fn one_through_four_are_valid_and_one_is_highest() {
    assert_eq!(Priority::new(1).unwrap(), Priority::HIGHEST);
    assert_eq!(Priority::new(4).unwrap(), Priority::LOWEST);
    assert!(Priority::HIGHEST < Priority::LOWEST);
}

#[test]
fn zero_and_five_are_refused() {
    assert_eq!(Priority::new(0), Err(PriorityError(0)));
    assert_eq!(Priority::new(5), Err(PriorityError(5)));
}

#[test]
fn default_is_lowest() {
    assert_eq!(Priority::default(), Priority::LOWEST);
}
