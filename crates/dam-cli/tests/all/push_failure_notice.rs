use crate::support;

use support::config_files::Remote;
use support::sandbox::Sandbox;

fn push_failed_notices(sb: &Sandbox) -> Vec<serde_json::Value> {
    let status = support::status_json(sb, &["--json"]);
    status["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "push_failed")
        .cloned()
        .collect()
}

#[test]
fn a_push_that_succeeds_clears_the_notice_of_its_earlier_failure() {
    let _guard = support::guard("a_push_that_succeeds_clears_the_notice_of_its_earlier_failure");
    let sb = Sandbox::new();
    sb.new_object(&["buy oat milk"]);
    sb.commit_everything();
    sb.fail_next_push();
    sb.dam(&["push"]);
    assert_eq!(push_failed_notices(&sb).len(), 1);

    let (ok, _, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert_eq!(push_failed_notices(&sb), Vec::<serde_json::Value>::new());
}

#[test]
fn a_push_refused_twice_leaves_one_notice() {
    let _guard = support::guard("a_push_refused_twice_leaves_one_notice");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ann"]);
    sb.commit_everything();
    sb.refuse_the_next_push();
    sb.dam(&["push"]);
    sb.refuse_the_next_push();
    sb.dam(&["push"]);
    assert_eq!(push_failed_notices(&sb).len(), 1);
}
