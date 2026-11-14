use super::*;

#[test]
fn a_group_name_is_taken_without_the_space_around_it() {
    assert_eq!(
        child_disposition("into:  leftovers  "),
        Ok(ChildDisposition::Into("leftovers".into()))
    );
}

#[test]
fn a_blank_group_name_is_refused_since_it_would_name_a_whitespace_group_task() {
    for blank in ["into:", "into: ", "into:\t", "into:   "] {
        assert!(child_disposition(blank).is_err(), "{blank:?}");
    }
}
