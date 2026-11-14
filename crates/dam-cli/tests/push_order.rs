mod support;

use support::sandbox::Sandbox;

struct Family {
    parent: String,
    child: String,
}

fn a_pushed_parent_sorting_before_its_child(sb: &Sandbox) -> Family {
    let first = sb.new_object(&["errands"]);
    let second = sb.new_object(&["chores"]);
    let (parent, child) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    let (ok, out, _) = sb.dam(&["show", &parent]);
    assert!(ok);
    let subject = if out.contains("errands") {
        "errands/"
    } else {
        "chores/"
    };
    assert!(sb.dam(&["mv", &child, subject]).0);
    commit_and_push(sb, "family");
    Family { parent, child }
}

fn commit_and_push(sb: &Sandbox, message: &str) -> (bool, String) {
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", message]).0);
    let (ok, out, _) = sb.dam(&["push"]);
    (ok, out)
}

fn sent_ops(push: &serde_json::Value) -> Vec<(String, String)> {
    push["mutations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            let short = m["oid"].as_str().unwrap()[..7].to_string();
            (short, m["op"].as_str().unwrap().to_string())
        })
        .collect()
}

fn last_two_calls(sb: &Sandbox) -> Vec<(String, String)> {
    let pushes = sb.pushes();
    pushes[pushes.len() - 2..]
        .iter()
        .flat_map(sent_ops)
        .collect()
}

fn child_moved_then_parent_removed(family: &Family) -> Vec<(String, String)> {
    vec![
        (family.child.clone(), "update".to_string()),
        (family.parent.clone(), "delete".to_string()),
    ]
}

#[test]
fn removing_a_parent_pushes_its_child_moving_up_before_the_removal() {
    let _guard = support::guard("removing_a_parent_pushes_its_child_moving_up_before_the_removal");
    let sb = Sandbox::new();
    let family = a_pushed_parent_sorting_before_its_child(&sb);
    let (ok, out, err) = sb.dam(&["rm", &family.parent]);
    assert!(ok, "{err}");
    assert!(out.contains("moved up"), "{out}");
    let (ok, out) = commit_and_push(&sb, "rm parent");
    assert!(ok && out.contains("fake: 2 sent, 2 ok"), "{out}");
    assert_eq!(
        last_two_calls(&sb),
        child_moved_then_parent_removed(&family)
    );
}

#[test]
fn a_refused_move_holds_the_removal_until_a_later_push_sends_it() {
    let _guard = support::guard("a_refused_move_holds_the_removal_until_a_later_push_sends_it");
    let sb = Sandbox::new();
    let family = a_pushed_parent_sorting_before_its_child(&sb);
    assert!(sb.dam(&["rm", &family.parent]).0);
    sb.fail_the_first_mutation_of_the_next_push();
    let (_, out) = commit_and_push(&sb, "rm parent");
    assert!(out.contains("fake: 1 sent, 0 ok, 2 failed"), "{out}");
    let held = sb.pushes().pop().unwrap();
    assert_eq!(
        sent_ops(&held),
        vec![(family.child.clone(), "update".to_string())]
    );
    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 2 sent, 2 ok"), "{out}");
    assert_eq!(
        last_two_calls(&sb),
        child_moved_then_parent_removed(&family)
    );
}
