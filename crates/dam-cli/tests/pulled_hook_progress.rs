mod support;

use serde_json::json;
use support::sandbox::Sandbox;
use support::upstream::{answer_pull_with, task};

const SPINNING_ONCE_PAST_TWO_SECONDS: &str = "function()\n  \
    if spun then return end\n  \
    spun = true\n  \
    local until_ = os.clock() + 2.1\n  \
    while os.clock() < until_ do end\n\
  end";

fn hooked_with_two_tasks_to_pull() -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!(
        "return {{\n  remote = {{ fake = {{ url = 'fake::' }} }},\n  \
         hook = {{ task_pulled = {SPINNING_ONCE_PAST_TWO_SECONDS} }},\n}}\n"
    ));
    answer_pull_with(&sb, json!({"objects": [], "removed": [], "sync": "t1"}));
    assert!(sb.output(&["pull"]).status.success());
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "call Bob", None), task("i:2", "pay rent", None)],
               "removed": [], "sync": "t2"}),
    );
    sb
}

#[test]
fn a_pulled_hook_past_two_seconds_says_once_in_plain_text_what_it_is_running() {
    let _guard =
        support::guard("a_pulled_hook_past_two_seconds_says_once_in_plain_text_what_it_is_running")
            .allow_slow("the line shows only after two seconds of hook run");
    let sb = hooked_with_two_tasks_to_pull();
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    assert_eq!(err, "dam: running task_pulled for 2 tasks\n");
}

#[test]
fn a_pulled_hook_past_two_seconds_says_nothing_under_json() {
    let _guard = support::guard("a_pulled_hook_past_two_seconds_says_nothing_under_json")
        .allow_slow("the line would show only after two seconds of hook run");
    let sb = hooked_with_two_tasks_to_pull();
    let (ok, _, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(err, "");
}
