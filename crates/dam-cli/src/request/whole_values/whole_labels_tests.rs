use std::ffi::OsString;

use dam_domain::Oid;

use crate::args::{Cli, Command, TokenArgs, TokenCommand};
use crate::request::{hook_edit_args, parse};

fn requested(words: &[&str], name: &str, json: &str) -> Cli {
    let file = std::env::temp_dir().join(format!(
        "dam-whole-labels-{name}-{}.json",
        std::process::id()
    ));
    std::fs::write(&file, json).unwrap();
    let line = ["dam"]
        .iter()
        .chain(words)
        .chain(&["--request"])
        .map(OsString::from)
        .chain([file.clone().into_os_string()])
        .collect();
    let cli = parse(line).ok().unwrap();
    std::fs::remove_file(&file).unwrap();
    cli
}

#[test]
fn an_edit_request_keeps_a_label_that_holds_a_comma_whole() {
    let task = Oid::parse(&"ab".repeat(20)).unwrap();
    let args = hook_edit_args(&task, r#"{"labels": ["a,b"], "unlabels": ["c, d"]}"#).unwrap();
    assert_eq!(args.labels, ["a,b"]);
    assert_eq!(args.unlabels, ["c, d"]);
}

#[test]
fn a_new_request_keeps_a_label_that_holds_a_comma_whole() {
    match requested(
        &["new"],
        "new",
        r#"{"subject": "x", "labels": ["a,b", "c"]}"#,
    )
    .command
    {
        Command::New(a) => assert_eq!(a.labels, ["a,b", "c"]),
        _ => panic!("wrong command"),
    }
}

#[test]
fn a_token_request_keeps_a_label_that_holds_a_comma_whole() {
    let json = r#"{"name": "bot", "allow": ["task:read"], "labels": ["a,b"]}"#;
    match requested(&["token", "create"], "token", json).command {
        Command::Token(TokenArgs {
            command: TokenCommand::Create(a),
        }) => assert_eq!(a.labels, ["a,b"]),
        _ => panic!("wrong command"),
    }
}
