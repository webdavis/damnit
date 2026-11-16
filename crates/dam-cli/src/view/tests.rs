use dam_domain::Object;

use super::{ObjectList, report};

#[test]
fn an_empty_list_says_so_on_standard_error_and_prints_no_row() {
    let objects: [Object; 0] = [];
    let listed = report(&ObjectList {
        objects: &objects,
        authors: &[],
        zone: &jiff::tz::TimeZone::UTC,
    })
    .unwrap();
    assert_eq!(listed.human, "");
    assert_eq!(listed.standard_error_lines, ["dam: no objects match"]);
}
