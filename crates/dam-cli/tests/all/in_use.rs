use crate::support;

use serde_json::json;
use support::fixture::run;
use support::sandbox::Sandbox;

fn holding_three_tasks() -> Sandbox {
    let sb = Sandbox::new();
    run(
        &sb,
        &[
            "new",
            "parse flags",
            "--path",
            "work/cli",
            "--labels",
            "urgent,bug",
        ],
    );
    run(
        &sb,
        &["new", "readme", "--path", "work", "--labels", "urgent"],
    );
    run(&sb, &["new", "weed", "--path", "home/garden"]);
    sb
}

fn printed(sb: &Sandbox, args: &[&str]) -> (String, serde_json::Value) {
    let (ok, text, err) = sb.dam(args);
    assert!(ok, "{err}");
    let (ok, answer, err) = sb.dam(&[args, &["-o", "json"]].concat());
    assert!(ok, "{err}");
    (text, serde_json::from_str(&answer).unwrap())
}

#[test]
fn dam_label_list_prints_each_label_the_store_holds_with_its_count() {
    let _guard = support::guard("dam_label_list_prints_each_label_the_store_holds_with_its_count");
    let (text, answer) = printed(&holding_three_tasks(), &["label", "list"]);
    assert_eq!(text, "bug     1\nurgent  2\n");
    assert_eq!(
        answer,
        json!({"labels": [{"name": "bug", "count": 1}, {"name": "urgent", "count": 2}]})
    );
}

#[test]
fn dam_path_list_prints_each_path_the_store_holds_and_its_parents_with_counts() {
    let _guard = support::guard(
        "dam_path_list_prints_each_path_the_store_holds_and_its_parents_with_counts",
    );
    let (text, answer) = printed(&holding_three_tasks(), &["path", "list"]);
    assert_eq!(
        text,
        "home/         1\nhome/garden/  1\nwork/         2\nwork/cli/     1\n"
    );
    assert_eq!(
        answer,
        json!({"paths": [
            {"path": "home/", "count": 1},
            {"path": "home/garden/", "count": 1},
            {"path": "work/", "count": 2},
            {"path": "work/cli/", "count": 1},
        ]})
    );
}
