use crate::support;

use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::withholding_helper::listed_task;

fn ann_pushed_then_renamed() -> (Sandbox, String) {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ann"]);
    sb.commit_everything();
    let ann = sb.oid_of("Ann");
    sb.answer_the_next_push();
    sb.run(&["push"]);
    sb.run(&["edit", &ann[..7], "--subject", "Ann 2"]);
    sb.commit_everything();
    (sb, ann)
}

fn two_pushes_answered(sb: &Sandbox) -> Vec<(String, String)> {
    let before = sb.pushes().len();
    sb.answer_the_next_push();
    sb.run(&["push"]);
    sb.answer_the_next_push();
    sb.run(&["push"]);
    sb.mutations_since(before)
}

#[test]
fn a_task_removed_here_but_not_committed_and_removed_in_the_app_is_not_sent_back() {
    let _guard = support::guard(
        "a_task_removed_here_but_not_committed_and_removed_in_the_app_is_not_sent_back",
    );
    let (sb, ann) = ann_pushed_then_renamed();
    sb.dam(&["push"]);
    sb.run(&["rm", &ann[..7]]);
    sb.answer_the_next_pull(&[], &[format!("r-{ann}")]);
    sb.run(&["pull"]);

    let before = sb.pushes().len();
    sb.answer_the_next_push();
    sb.run(&["push"]);
    sb.commit_everything();
    sb.answer_the_next_push();
    sb.run(&["push"]);

    assert_eq!(sb.mutations_since(before), vec![]);
}

#[test]
fn a_task_whose_rename_was_refused_and_then_removed_in_the_app_is_not_sent_back() {
    let _guard = support::guard(
        "a_task_whose_rename_was_refused_and_then_removed_in_the_app_is_not_sent_back",
    );
    let (sb, ann) = ann_pushed_then_renamed();
    sb.refuse_the_next_push();
    sb.dam(&["push"]);
    sb.answer_the_next_pull(&[], &[format!("r-{ann}")]);
    sb.run(&["pull"]);

    assert_eq!(two_pushes_answered(&sb), vec![]);
}

#[test]
fn a_parent_deleted_here_after_its_subtask_was_removed_in_the_app_is_deleted_upstream() {
    let _guard = support::guard(
        "a_parent_deleted_here_after_its_subtask_was_removed_in_the_app_is_deleted_upstream",
    );
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ada"]);
    sb.new_object(&["Ann", "--path", "Ada/"]);
    sb.commit_everything();
    let (ada, ann) = (sb.oid_of("Ada"), sb.oid_of("Ann"));
    sb.answer_the_next_push();
    sb.run(&["push"]);
    sb.run(&["edit", &ann[..7], "--subject", "Ann 2"]);
    sb.commit_everything();
    sb.dam(&["push"]);
    let ada_listed = listed_task(&format!("r-{ada}"), "Ada");
    sb.answer_the_next_pull(&[ada_listed], &[format!("r-{ann}")]);
    sb.run(&["pull"]);
    sb.run(&["rm", &ann[..7]]);
    sb.run(&["rm", &ada[..7]]);
    sb.commit_everything();

    let sent = two_pushes_answered(&sb);

    assert_eq!(sent, vec![("delete".to_string(), ada)]);
    let (_, status, _) = sb.dam(&["status"]);
    assert!(!status.contains("still under it upstream"), "{status}");
}
