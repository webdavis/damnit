use crate::support;

use support::config_files::Remote;
use support::sandbox::Sandbox;

fn sent_in_the_push(sb: &Sandbox, index: usize) -> Vec<(String, String, String)> {
    let pushes = sb.pushes();
    let mutations = pushes[index]["mutations"].as_array().cloned().unwrap();
    let mut sent: Vec<_> = mutations
        .iter()
        .map(|m| {
            let field = |k: &str| m[k].as_str().unwrap_or_default().to_string();
            let subject = m["object"]["subject"].as_str().unwrap_or_default();
            (field("op"), field("oid"), subject.to_string())
        })
        .collect();
    sent.sort();
    sent
}

fn update(oid: &str, subject: &str) -> (String, String, String) {
    ("update".into(), oid.into(), subject.into())
}

fn push_withholding(sb: &Sandbox, oids: &[&str]) {
    sb.withhold_answers_for(oids);
    sb.answer_the_next_push();
    sb.dam(&["push"]);
}

#[test]
fn a_push_after_a_partly_answered_commit_never_sends_an_older_subject() {
    let _guard =
        support::guard("a_push_after_a_partly_answered_commit_never_sends_an_older_subject");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ann"]);
    sb.new_object(&["Bob"]);
    sb.commit_everything();
    let (ann, bob) = (sb.oid_of("Ann"), sb.oid_of("Bob"));
    push_withholding(&sb, &[]);
    sb.run(&["edit", &ann[..7], "--subject", "Ann 2"]);
    sb.run(&["edit", &bob[..7], "--subject", "Bob 2"]);
    sb.commit_everything();
    push_withholding(&sb, &[&bob]);
    sb.run(&["edit", &ann[..7], "--subject", "Ann 3"]);
    sb.commit_everything();
    push_withholding(&sb, &[&bob]);
    let mut ann_edited_and_bob_resent = vec![update(&ann, "Ann 3"), update(&bob, "Bob 2")];
    ann_edited_and_bob_resent.sort();
    assert_eq!(sent_in_the_push(&sb, 2), ann_edited_and_bob_resent);

    push_withholding(&sb, &[]);

    assert_eq!(sent_in_the_push(&sb, 3), vec![update(&bob, "Bob 2")]);
    assert_eq!(sb.oid_of("Ann 3"), ann);
    let pushes_so_far = sb.pushes().len();
    push_withholding(&sb, &[]);
    assert!(sb.mutations_since(pushes_so_far).is_empty());
}
