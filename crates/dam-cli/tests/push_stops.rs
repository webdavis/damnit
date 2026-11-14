mod support;

use support::sandbox::Sandbox;

fn two_committed_tasks(sb: &Sandbox) {
    for subject in ["buy oat milk", "call Ada"] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", subject]).0);
    }
}

fn rate_limited_push(after_the_first: bool) -> (Sandbox, std::process::Output) {
    let sb = Sandbox::new();
    two_committed_tasks(&sb);
    sb.rate_limit_the_next_push(after_the_first);
    let out = sb.output(&["push"]);
    (sb, out)
}

#[test]
fn a_rate_limit_after_the_first_answer_reads_as_any_rate_limit_and_keeps_that_answer() {
    let _guard = support::guard(
        "a_rate_limit_after_the_first_answer_reads_as_any_rate_limit_and_keeps_that_answer",
    );
    let (_, refused_whole) = rate_limited_push(false);
    let (sb, stopped_partway) = rate_limited_push(true);

    assert_eq!(stopped_partway.status.code(), refused_whole.status.code());
    assert_ne!(stopped_partway.status.code(), Some(0));
    assert_eq!(stopped_partway.stdout, refused_whole.stdout);
    assert_eq!(stopped_partway.stderr, refused_whole.stderr);
    let said = String::from_utf8_lossy(&stopped_partway.stderr);
    assert!(said.contains("retry after 42s"), "{said}");

    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let resent = sb.pushes().pop().unwrap();
    let sent_first = &sb.pushes()[0]["mutations"];
    assert_eq!(resent["mutations"].as_array().unwrap().len(), 1);
    assert_eq!(resent["mutations"][0], sent_first[1]);
}
