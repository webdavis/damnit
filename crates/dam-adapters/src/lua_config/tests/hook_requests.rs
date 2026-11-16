use dam_application::FunctionFault;
use serde_json::{Value, json};

use super::hook_runs::run;

fn document(task_done: &str) -> Value {
    let request = run(task_done).unwrap().unwrap();
    serde_json::from_str(&request.json).unwrap()
}

#[test]
fn a_table_a_hook_returns_comes_back_as_its_edit_request() {
    let request = run("function() return { labels = { 'errand' } } end")
        .unwrap()
        .unwrap();
    assert!(
        request.file.ends_with("hooks/task_done.lua"),
        "{:?}",
        request.file
    );
    assert_eq!(request.line, Some(1));
    assert_eq!(
        serde_json::from_str::<Value>(&request.json).unwrap(),
        json!({ "labels": ["errand"] })
    );
}

#[test]
fn every_kind_of_value_a_request_takes_comes_back_as_json() {
    assert_eq!(
        document(
            "function() return { subject = 'pay', priority = 2, no_done = true, \
             unlabels = { 'a', 'b' }, estimate = 1.5 } end"
        ),
        json!({
            "subject": "pay",
            "priority": 2,
            "no_done": true,
            "unlabels": ["a", "b"],
            "estimate": 1.5,
        })
    );
}

#[test]
fn an_empty_table_comes_back_as_an_empty_request() {
    assert_eq!(document("function() return {} end"), json!({}));
}

#[test]
fn a_table_holding_a_function_fails_as_a_wrong_request() {
    let failure = run("function() return { subject = print } end").unwrap_err();
    assert_eq!(
        failure.fault,
        FunctionFault::Requested {
            why: "subject is a function, which no request takes".into()
        }
    );
}
