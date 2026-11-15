use crate::support;

use support::sandbox::Sandbox;

const NOTHING_OWED: [&str; 2] = ["  ✓ pushes: 1 remote: 1 ok", "    ✓ fake: nothing owed"];

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
