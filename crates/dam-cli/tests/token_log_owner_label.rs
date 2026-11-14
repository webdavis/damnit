mod support;

use support::hidden::text;
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

#[test]
fn the_owners_commits_read_you_to_the_owner_and_owner_to_a_token() {
    let _guard = support::guard("the_owners_commits_read_you_to_the_owner_and_owner_to_a_token");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    let (ok, _, err) = sb.dam(&["add", "-A"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "seed"]);
    assert!(ok, "{err}");
    let held = grant_to(&sb, "tool", 1, vec![grant(&["history:read"])]);
    assert!(
        text(&sb.output(&["log"]).stdout)
            .trim_end()
            .ends_with("you  seed")
    );
    let log = text(&sb.output_holding(&held, &["log"]).stdout);
    assert!(log.trim_end().ends_with("owner  seed"), "{log}");
}
