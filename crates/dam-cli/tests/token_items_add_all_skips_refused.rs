mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn dam(sb: &Sandbox, line: &[&str]) {
    let (ok, _, err) = sb.dam(line);
    assert!(ok, "{err}");
}

#[test]
fn add_all_leaves_a_rename_that_cannot_name_a_folder_waiting_and_takes_the_rest() {
    let _guard = support::guard(
        "add_all_leaves_a_rename_that_cannot_name_a_folder_waiting_and_takes_the_rest",
    );
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    dam(&sb, &["add", "-A"]);
    dam(&sb, &["commit", "-m", "seed"]);
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&["task:read", "task:change", "task:create"])],
    );
    let line = ["edit", &parent, "--subject", ".", "--json"];
    let rename = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let line = ["new", "Call the dentist", "--json"];
    let dentist = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    sb.new_object(&["child", "--path", "parent/"]);

    dam(&sb, &["add", "-A"]);

    let (ok, status, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    let waiting: Vec<&str> = status["waiting"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["hash"].as_str().unwrap())
        .collect();
    assert_eq!(waiting, vec![rename["hash"].as_str().unwrap()]);
    let staged = status["staged"].to_string();
    assert!(
        staged.contains(dentist["hash"].as_str().unwrap()),
        "{staged}"
    );
}
