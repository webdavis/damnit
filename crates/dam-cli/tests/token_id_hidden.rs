mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant_to, refused};

fn send(sb: &Sandbox, held: &str, id: &str) -> serde_json::Value {
    let line = [
        "new",
        "Call dentist",
        "--path",
        "work",
        "--id",
        id,
        "--json",
    ];
    answer(&sb.output_holding(held, &line), "sent.schema.json")
}

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn taken(sb: &Sandbox, held: &str, id: &str) -> String {
    let oid = send(sb, held, id)["oid"].as_str().unwrap().to_string();
    owner(sb, &["add", &oid]);
    owner(sb, &["commit", "-m", "take it"]);
    oid
}

fn without_its_identity(mut sent: serde_json::Value) -> serde_json::Value {
    for key in ["hash", "oid", "created", "id"] {
        sent.as_object_mut().unwrap().remove(key);
    }
    sent
}

#[test]
fn a_repeat_for_a_task_moved_out_of_the_grant_answers_as_for_a_removed_one() {
    let _guard =
        support::guard("a_repeat_for_a_task_moved_out_of_the_grant_answers_as_for_a_removed_one");
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "vpt",
        1,
        "[[token.vpt.grant]]\nallow = [\"task:read\", \"task:create\", \"task:change\"]\npaths = [\"work\"]\n",
    );
    let hidden = taken(&sb, &held, "hidden");
    let removed = taken(&sb, &held, "removed");
    owner(&sb, &["mv", &hidden, "home"]);
    owner(&sb, &["rm", &removed]);

    let while_hidden = send(&sb, &held, "hidden");
    let once_removed = send(&sb, &held, "removed");
    assert_eq!(while_hidden["answer"], "new");
    assert_ne!(while_hidden["oid"], hidden.as_str());
    assert_eq!(
        without_its_identity(while_hidden),
        without_its_identity(once_removed)
    );
}

fn moved_plan(with_hidden_child: bool) -> std::process::Output {
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "vpt",
        1,
        "[[token.vpt.grant]]\nallow = [\"task:read\", \"task:create\", \"task:change\", \"task:move\"]\nlabels = [\"vpt\"]\n",
    );
    let line = ["new", "Plan", "--id", "plan", "--label", "vpt", "--json"];
    let oid = answer(&sb.output_holding(&held, &line), "sent.schema.json")["oid"]
        .as_str()
        .unwrap()
        .to_string();
    owner(&sb, &["add", &oid]);
    owner(&sb, &["commit", "-m", "take it"]);
    if with_hidden_child {
        owner(&sb, &["new", "Step", "--path", "Plan"]);
    }
    let moving = [&line[..6], &["--path", "inbox", "--json"]].concat();
    sb.output_holding(&held, &moving)
}

#[test]
fn a_repeat_that_moves_a_task_answers_the_same_with_a_hidden_child() {
    let _guard = support::guard("a_repeat_that_moves_a_task_answers_the_same_with_a_hidden_child");
    let alone = moved_plan(false);
    let beside_a_hidden_child = moved_plan(true);
    assert_eq!(refused(&alone), "not_open_to_tokens");
    assert_eq!(alone.stdout, beside_a_hidden_child.stdout);
    assert_eq!(alone.stderr, beside_a_hidden_child.stderr);
}
