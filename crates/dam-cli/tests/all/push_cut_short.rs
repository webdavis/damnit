use crate::support;

use support::config_files::Remote;
use support::sandbox::Sandbox;

fn two_committed_tasks_on_a_long_deadline() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("30s"));
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
fn a_push_cancelled_after_the_first_answer_says_how_much_it_kept_and_resends_only_the_rest() {
    let _guard = support::guard(
        "a_push_cancelled_after_the_first_answer_says_how_much_it_kept_and_resends_only_the_rest",
    );
    let sb = two_committed_tasks_on_a_long_deadline();
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
