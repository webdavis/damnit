mod support;

use support::config_files::Remote;
use support::errors::error_document;
use support::sandbox::Sandbox;

const A_SECOND_IS_THE_SHORTEST_DEADLINE_THE_CONFIG_TAKES: &str =
    "a push cut short by the deadline waits out the shortest deadline the config takes, 1s";

fn two_committed_tasks_on_a_one_second_deadline() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("1s"));
    for subject in ["buy oat milk", "call Ada"] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", subject]).0);
    }
    sb
}

fn only_the_second_is_sent_again(sb: &Sandbox) {
    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let resent = sb.pushes().pop().unwrap();
    assert_eq!(resent["mutations"].as_array().unwrap().len(), 1);
    assert_eq!(resent["mutations"][0], sb.pushes()[0]["mutations"][1]);
}

#[test]
fn a_push_the_deadline_cuts_short_says_how_much_it_kept_and_resends_only_the_rest() {
    let _guard = support::guard(
        "a_push_the_deadline_cuts_short_says_how_much_it_kept_and_resends_only_the_rest",
    )
    .allow_slow(A_SECOND_IS_THE_SHORTEST_DEADLINE_THE_CONFIG_TAKES);
    let sb = two_committed_tasks_on_a_one_second_deadline();
    sb.hang_the_next_push(true);

    let out = sb.output(&["push"]);

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: helper fake gave no answer within 1s; 1 kept, 1 unanswered\n"
    );
    only_the_second_is_sent_again(&sb);
}

#[test]
fn a_push_the_deadline_cuts_short_before_any_answer_prints_as_it_always_did() {
    let _guard =
        support::guard("a_push_the_deadline_cuts_short_before_any_answer_prints_as_it_always_did")
            .allow_slow(A_SECOND_IS_THE_SHORTEST_DEADLINE_THE_CONFIG_TAKES);
    let sb = two_committed_tasks_on_a_one_second_deadline();
    sb.hang_the_next_push(false);

    let out = sb.output(&["push"]);

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: helper fake gave no answer within 1s\n"
    );
    assert_eq!(sb.pushes().len(), 1);
}

#[test]
fn the_json_error_of_a_cut_short_push_carries_the_counts_in_its_message() {
    let _guard =
        support::guard("the_json_error_of_a_cut_short_push_carries_the_counts_in_its_message")
            .allow_slow(A_SECOND_IS_THE_SHORTEST_DEADLINE_THE_CONFIG_TAKES);
    let sb = two_committed_tasks_on_a_one_second_deadline();
    sb.hang_the_next_push(true);

    let out = sb.output(&["push", "--json"]);

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        error_document(&out),
        serde_json::json!({"error": {
            "kind": "helper",
            "rule": null,
            "message": "helper fake gave no answer within 1s; 1 kept, 1 unanswered",
            "oids": [],
        }})
    );
}

#[test]
fn a_push_cancelled_after_the_first_answer_says_how_much_it_kept_and_resends_only_the_rest() {
    let _guard = support::guard(
        "a_push_cancelled_after_the_first_answer_says_how_much_it_kept_and_resends_only_the_rest",
    );
    let sb = two_committed_tasks_on_a_one_second_deadline();
    sb.add_remote("fake", Remote::fake().deadline("30s"));
    sb.hang_the_next_push(true);

    let pushing = sb.spawn(&["push"]);
    sb.until_the_push_hangs();
    let interrupted = std::process::Command::new("kill")
        .args(["-INT", &pushing.id().to_string()])
        .status()
        .unwrap();
    assert!(interrupted.success());
    let out = pushing.wait_with_output().unwrap();

    assert_eq!(out.status.code(), Some(3));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: cancelled; 1 kept, 1 unanswered\n"
    );
    only_the_second_is_sent_again(&sb);
}
