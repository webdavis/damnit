use std::ffi::OsString;

use super::*;
use crate::args::Command;
use crate::request::parse;

fn by_request(document: &str) -> Command {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), document).unwrap();
    let line = ["dam", "edit", "--request"]
        .map(OsString::from)
        .into_iter()
        .chain([file.path().as_os_str().to_owned()])
        .collect();
    match parse(line) {
        Ok(cli) => cli.command,
        Err(refused) => panic!("{}", refused.error),
    }
}

#[test]
fn a_request_names_several_oids_as_an_array() {
    let (ctx, oids) = three_tasks();
    let document = serde_json::json!({"oid": [&oids[0], &oids[1]], "subject": "x"});
    let out = dispatch_a_verb_that_opens_the_store(&ctx, by_request(&document.to_string()));
    assert_eq!(last_line(&out.unwrap()), "changed 2 of 2");
}

#[test]
fn a_request_carries_the_filter_preview_and_expect() {
    let (ctx, _) = three_tasks();
    let document = r#"{"filter": "work", "preview": true, "expect": 2, "subject": "x"}"#;
    let out = dispatch_a_verb_that_opens_the_store(&ctx, by_request(document)).unwrap();
    assert_eq!(out.data["preview"], true);
    assert_eq!(out.data["count"], 2);
}
