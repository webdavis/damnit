use super::super::untouched::{AtThePull, Untouched};

use Untouched::{Conflict, LeftAlone};

#[test]
fn only_uncommitted_work_the_app_changed_and_the_pull_would_overwrite_is_a_conflict() {
    let rows = [
        (false, false, false, None),
        (false, false, true, None),
        (false, true, false, None),
        (false, true, true, None),
        (true, false, false, Some(LeftAlone)),
        (true, false, true, Some(LeftAlone)),
        (true, true, false, None),
        (true, true, true, Some(Conflict)),
    ];
    for (uncommitted, changed_in_the_app, the_pull_would_write, expected) in rows {
        let facts = AtThePull {
            uncommitted,
            changed_in_the_app,
            the_pull_would_write,
        };
        assert_eq!(
            Untouched::of(facts),
            expected,
            "uncommitted {uncommitted}, changed in the app {changed_in_the_app}, \
             the pull would write {the_pull_would_write}"
        );
    }
}
