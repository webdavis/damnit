mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn owner(sb: &Sandbox, args: &[&str]) -> String {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
    out
}

fn removal_of_a_task_the_owner_then_removed(committed: bool) -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let rights = ["task:create", "task:read", "task:remove"];
    let held = grant_to(&sb, "tool", 1, vec![grant(&rights)]);
    let line = ["new", "Call the dentist", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let oid = sent["oid"].as_str().unwrap().to_string();
    owner(&sb, &["add", &oid]);
    owner(&sb, &["commit", "-m", "dentist"]);
    let removal = sb.output_holding(&held, &["rm", &oid, "--json"]);
    let item = answer(&removal, "removal.schema.json")["item"]
        .as_str()
        .unwrap()
        .to_string();
    owner(&sb, &["rm", &oid]);
    if committed {
        owner(&sb, &["add", &oid]);
        owner(&sb, &["commit", "-m", "gone"]);
    }
    (sb, item, oid)
}

#[test]
fn adding_a_removal_of_a_task_already_gone_settles_it_and_says_so() {
    let _guard = support::guard("adding_a_removal_of_a_task_already_gone_settles_it_and_says_so");
    for committed in [true, false] {
        let (sb, item, oid) = removal_of_a_task_the_owner_then_removed(committed);
        let added = answer(&sb.output(&["add", &item, "--json"]), "add.schema.json");
        assert_eq!(
            added["gone"],
            serde_json::json!([{"item": item, "oid": oid}]),
            "{committed}"
        );
        let status = support::status_json(&sb, &["--json"]);
        assert_eq!(status["waiting"], serde_json::json!([]), "{committed}");
        assert!(
            status["staged"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["from"].is_null()),
            "{status}"
        );
    }
}

#[test]
fn the_text_says_the_task_is_already_gone() {
    let _guard = support::guard("the_text_says_the_task_is_already_gone");
    let (sb, item, oid) = removal_of_a_task_the_owner_then_removed(true);
    assert_eq!(
        owner(&sb, &["add", &item]).trim_end(),
        format!(
            "{} removes {}, which is already gone; the item is settled",
            &item[..7],
            &oid[..7]
        )
    );
}

#[test]
fn declining_a_removal_of_a_task_already_gone_still_declines_it() {
    let _guard = support::guard("declining_a_removal_of_a_task_already_gone_still_declines_it");
    let (sb, item, _) = removal_of_a_task_the_owner_then_removed(true);
    let out = sb.output(&["rm", &item, "--json"]);
    assert_eq!(answer(&out, "settled.schema.json")["outcome"], "declined");
}
