use super::*;

#[test]
fn every_field_round_trips_through_its_name() {
    for field in Field::ALL {
        assert_eq!(Field::parse(field.as_str()), Some(field), "{field}");
    }
}

#[test]
fn every_name_is_distinct() {
    let mut names: Vec<&str> = Field::ALL.iter().map(|f| f.as_str()).collect();
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(names.len(), count, "two fields share a name");
}

#[test]
fn an_unknown_name_is_none_so_a_field_from_a_newer_protocol_is_ignored_not_refused() {
    assert_eq!(Field::parse("eventtype"), None);
    assert_eq!(Field::parse(""), None);
}
