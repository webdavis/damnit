use super::tables::{either, one, table};
use crate::select::Filter;

#[test]
fn a_filter_names_every_saved_filter_it_reaches() {
    let t = either(
        table(&[
            ("filter", one("not:hot,today")),
            ("parent", one("in:projects")),
            ("due", one("on:standup")),
            ("start", one("on:thu")),
        ]),
        vec![table(&[("attached", one("in:meetings"))])],
    );
    let filter = Filter::read(&t).unwrap();
    let mut names = filter.saved_filters();
    names.sort();
    assert_eq!(names, ["hot", "meetings", "projects", "standup", "today"]);
}
