use crate::support;

use support::config_files::Remote;
use support::doctor::listing;
use support::sandbox::Sandbox;

const PUSH: &str = "      → run dam push";
const NOTHING_OWED: [&str; 2] = ["  ✓ pushes: 1 remote: 1 ok", "    ✓ fake: nothing owed"];
const A_PUSH_CUT_SHORT_WAITS_OUT_ITS_DEADLINE: &str =
    "a push cut short waits out a two second deadline a busy machine answers within";

fn rows(sb: &Sandbox) -> (Option<i32>, Vec<String>) {
    let answer = sb.doctor(&["doctor", "pushes"]);
    let rows = answer
        .out
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    (answer.code, rows)
}

fn commit_two_tasks(sb: &Sandbox) {
    for subject in ["buy oat milk", "call Ada"] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", subject]).0);
    }
}

#[test]
fn no_store_owes_nothing() {
    let _guard = support::guard("no_store_owes_nothing");
    let sb = Sandbox::new();
    assert_eq!(
        rows(&sb),
        (Some(0), NOTHING_OWED.map(String::from).to_vec())
    );
}

#[test]
fn unpushed_commits_are_a_warning_read_without_a_write() {
    let _guard = support::guard("unpushed_commits_are_a_warning_read_without_a_write");
    let sb = Sandbox::new();
    commit_two_tasks(&sb);
    let before = listing(sb.dir.path());
    let (code, said) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        said,
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 2 changes still owed",
            PUSH
        ]
    );
    assert_eq!(listing(sb.dir.path()), before);

    assert!(sb.dam(&["push"]).0);
    assert_eq!(rows(&sb).1, NOTHING_OWED);
}

#[test]
fn a_change_the_remote_refused_is_still_owed() {
    let _guard = support::guard("a_change_the_remote_refused_is_still_owed");
    let sb = Sandbox::new();
    commit_two_tasks(&sb);
    sb.fail_the_first_mutation_of_the_next_push();
    sb.dam(&["push"]);
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 1 change still owed",
            PUSH
        ]
    );
}

#[test]
fn a_change_sent_without_an_answer_is_a_warning() {
    let _guard = support::guard("a_change_sent_without_an_answer_is_a_warning")
        .allow_slow(A_PUSH_CUT_SHORT_WAITS_OUT_ITS_DEADLINE);
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("2s"));
    commit_two_tasks(&sb);
    sb.hang_the_next_push(true);
    sb.dam(&["push"]);
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 1 change still owed, 1 change sent without an answer",
            PUSH
        ]
    );
}

#[test]
fn pushes_are_not_checked_on_a_store_this_dam_does_not_read() {
    let _guard = support::guard("pushes_are_not_checked_on_a_store_this_dam_does_not_read");
    let sb = Sandbox::new();
    std::fs::write(sb.dir.path().join("dam.db"), "not a database ".repeat(40)).unwrap();
    assert_eq!(
        rows(&sb),
        (
            Some(0),
            vec!["  · pushes: not checked, the database did not open".into()]
        )
    );
}

#[test]
fn pushes_are_not_checked_without_a_config_or_a_remote() {
    let _guard = support::guard("pushes_are_not_checked_without_a_config_or_a_remote");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    assert_eq!(
        rows(&sb).1,
        ["  · pushes: not checked, the config did not read"]
    );
    sb.write_config_lua("return {}\n");
    sb.write_secrets_lua("return { schema = 1 }\n");
    assert_eq!(rows(&sb).1, ["  · pushes: no remotes configured"]);
}

#[test]
fn pushes_are_not_checked_beside_a_side_file_the_database_check_warns_of() {
    let _guard =
        support::guard("pushes_are_not_checked_beside_a_side_file_the_database_check_warns_of");
    let sb = Sandbox::new();
    commit_two_tasks(&sb);
    std::fs::write(sb.dir.path().join("dam.db-wal"), b"").unwrap();
    assert_eq!(
        rows(&sb),
        (
            Some(0),
            vec!["  · pushes: not checked, the database was not read".into()]
        )
    );
}
