mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

const EVERY_RIGHT: [&str; 4] = ["task:create", "task:read", "task:change", "task:remove"];

fn tool() -> (Sandbox, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&EVERY_RIGHT)]);
    (sb, held)
}

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn sent(sb: &Sandbox, held: &str, line: &[&str]) -> serde_json::Value {
    let mut line = line.to_vec();
    line.push("--json");
    answer(&sb.output_holding(held, &line), "sent.schema.json")
}

fn taken_back(sb: &Sandbox, held: &str, named: &str) -> serde_json::Value {
    let out = sb.output_holding(held, &["rm", named, "--json"]);
    let document = answer(&out, "rm.schema.json");
    answer(&out, "removal.schema.json");
    document
}

fn withdrawn(item: &str, oid: &str) -> serde_json::Value {
    serde_json::json!({"answer": "withdrawn", "item": item, "oid": oid})
}

#[test]
fn a_new_item_named_by_oid_or_prefix_is_withdrawn_with_the_removal_answer() {
    let _guard =
        support::guard("a_new_item_named_by_oid_or_prefix_is_withdrawn_with_the_removal_answer");
    let (sb, held) = tool();
    for named in [
        |oid: &str| oid.to_string(),
        |oid: &str| oid[..7].to_string(),
    ] {
        let oid = sent(&sb, &held, &["new", "Call the dentist"])["oid"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(taken_back(&sb, &held, &named(&oid)), withdrawn(&oid, &oid));
    }
}

#[test]
fn a_change_or_removal_item_named_by_hash_is_withdrawn_with_its_task() {
    let _guard =
        support::guard("a_change_or_removal_item_named_by_hash_is_withdrawn_with_its_task");
    let (sb, held) = tool();
    let oid = sent(&sb, &held, &["new", "Call the dentist"])["oid"]
        .as_str()
        .unwrap()
        .to_string();
    owner(&sb, &["add", &oid]);
    owner(&sb, &["commit", "-m", "dentist"]);
    let change = sent(&sb, &held, &["edit", &oid, "--subject", "Call the vet"]);
    let hash = change["hash"].as_str().unwrap();
    assert_eq!(taken_back(&sb, &held, hash), withdrawn(hash, &oid));
    let removal = taken_back(&sb, &held, &oid);
    let hash = removal["item"].as_str().unwrap();
    assert_eq!(taken_back(&sb, &held, hash), withdrawn(hash, &oid));
    let status = support::status_json(&sb, &["--json"]);
    assert_eq!(status["waiting"], serde_json::json!([]));
}

#[test]
fn the_text_line_for_a_withdrawn_new_item_names_it_once() {
    let _guard = support::guard("the_text_line_for_a_withdrawn_new_item_names_it_once");
    let (sb, held) = tool();
    let oid = sent(&sb, &held, &["new", "Call the dentist"])["oid"]
        .as_str()
        .unwrap()
        .to_string();
    let out = sb.output_holding(&held, &["rm", &oid]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim_end(),
        format!("withdrawn  {}", &oid[..7])
    );
}

#[test]
fn parallel_withdraws_of_one_item_all_exit_0_and_one_says_withdrawn() {
    let _guard = support::guard("parallel_withdraws_of_one_item_all_exit_0_and_one_says_withdrawn");
    let (sb, held) = tool();
    sent(&sb, &held, &["new", "Fix the build", "--id", "w"]);
    let racers: Vec<_> = (0..12)
        .map(|_| {
            sb.command(&["rm", "--id", "w", "--json"])
                .env("DAM_TOKEN", &held)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let mut answers: Vec<String> = racers
        .into_iter()
        .map(|racer| {
            let out = racer.wait_with_output().unwrap();
            answer(&out, "removal.schema.json")["answer"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    answers.sort();
    let mut expected = vec!["nothing".to_string(); 11];
    expected.push("withdrawn".to_string());
    assert_eq!(answers, expected);
}
