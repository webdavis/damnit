mod support;

use support::hidden::text;
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

#[test]
fn the_owners_rows_read_you_to_the_owner_and_owner_to_a_token() {
    let _guard = support::guard("the_owners_rows_read_you_to_the_owner_and_owner_to_a_token");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "working:read"])]);
    let row = |out: &std::process::Output| {
        text(&out.stdout)
            .lines()
            .find(|l| l.contains("buy milk"))
            .unwrap()
            .to_string()
    };
    assert!(row(&sb.output(&["status"])).ends_with("buy milk  you"));
    assert!(row(&sb.output_holding(&held, &["status"])).ends_with("buy milk  owner"));
}
