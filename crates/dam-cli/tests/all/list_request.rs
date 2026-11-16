use crate::support;

use support::request::fed;
use support::sandbox::Sandbox;

#[test]
fn a_list_request_on_standard_input_selects_sorts_and_cuts_like_the_flags() {
    let _guard =
        support::guard("a_list_request_on_standard_input_selects_sorts_and_cuts_like_the_flags");
    let sb = Sandbox::new();
    sb.new_object(&["calm", "--path", "work/", "-p", "4"]);
    sb.new_object(&["urgent", "--path", "work/", "-p", "1", "--labels", "hot"]);
    sb.new_object(&["middle", "--path", "work/", "-p", "2", "--labels", "hot"]);
    sb.new_object(&["away", "--path", "home/", "-p", "1", "--labels", "hot"]);
    let request = r#"{"path": "work/", "any": [{"labels": "hot", "priority": "..2"}, {"subject": "calm"}], "sort": "priority:desc", "limit": 2}"#;
    let out = fed(&sb, &["list", "-o", "json", "--request", "-"], request);
    assert!(out.status.success(), "{out:?}");
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let subjects: Vec<&str> = listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["calm", "middle"]);
}
