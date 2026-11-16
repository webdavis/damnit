use std::ffi::OsString;

use dam_domain::Oid;

use super::hook_edit_args;
use crate::request::parse;

const WRONG: &str = r#"{"labels": "errand"}"#;

fn task() -> Oid {
    Oid::parse(&"ab".repeat(20)).unwrap()
}

fn refused_by_dam_edit_request(json: &str) -> String {
    let file = std::env::temp_dir().join(format!("dam-hook-request-{}.json", std::process::id()));
    std::fs::write(&file, json).unwrap();
    let line = ["dam", "edit", "--request"]
        .map(OsString::from)
        .into_iter()
        .chain([file.clone().into_os_string()])
        .collect();
    let refused = parse(line).err().unwrap();
    std::fs::remove_file(&file).unwrap();
    refused.error.to_string()
}

#[test]
fn a_hooks_request_is_refused_by_the_same_reader_as_dam_edit_request() {
    let refused = hook_edit_args(&task(), WRONG).unwrap_err().to_string();
    assert_eq!(refused, refused_by_dam_edit_request(WRONG));
    assert_eq!(refused, "--request: labels must be an array");
}

#[test]
fn a_hooks_request_reads_into_the_flags_of_dam_edit_for_its_task() {
    let args = hook_edit_args(&task(), r#"{"labels": ["errand"], "no_due": true}"#).unwrap();
    assert_eq!(args.oid, [task().to_string()]);
    assert_eq!(args.labels, ["errand"]);
    assert!(args.no_due);
}

#[test]
fn a_hooks_request_may_not_carry_meta() {
    let refused = hook_edit_args(&task(), r#"{"meta": {"session": "a"}}"#).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--request: meta is not a field a hook's request takes"
    );
}
