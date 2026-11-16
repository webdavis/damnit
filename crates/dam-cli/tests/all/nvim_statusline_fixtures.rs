use crate::nvim_fixtures::{answer, assert_fixture};
use crate::support;

use support::sandbox::Sandbox;

#[test]
fn the_list_overdue_fixture_is_what_dam_list_overdue_answers_for_one_overdue_task() {
    let _guard = support::guard(
        "the_list_overdue_fixture_is_what_dam_list_overdue_answers_for_one_overdue_task",
    );
    let sb = Sandbox::new();
    sb.new_object(&["renew the passport", "--due", "2026-09-14"]);
    sb.new_object(&["plant the bulbs", "--due", "2030-11-18"]);
    let text = answer(
        &sb,
        &["list", "--overdue", "-o", "json"],
        "list.schema.json",
    );
    assert_fixture("list-overdue.json", &text);
}
