use crate::nvim_fixtures::{answer, assert_fixture, error_document};
use crate::support;

use support::fixture;

const EXIT_USAGE: i32 = 2;

#[test]
fn the_filter_create_fixtures_are_what_the_filter_bar_reads_when_it_saves() {
    let _guard =
        support::guard("the_filter_create_fixtures_are_what_the_filter_bar_reads_when_it_saves");
    let sb = fixture::seeded();
    let create = [
        "filter",
        "create",
        "bugs",
        "--labels",
        "bug",
        "--overdue",
        "-o",
        "json",
    ];
    assert_fixture(
        "filter-create.json",
        &answer(&sb, &create, "filter-create.schema.json"),
    );
    let taken = sb.output(&create);
    assert_fixture(
        "error-filter-create-taken.json",
        &error_document(taken, EXIT_USAGE),
    );
}
