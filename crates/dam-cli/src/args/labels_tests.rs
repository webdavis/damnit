use clap::Parser;

use super::{Cli, Command, TokenArgs, TokenCommand};

fn parsed(argv: &[&str]) -> Cli {
    Cli::try_parse_from(argv).unwrap()
}

fn added_on_new(flags: &[&str]) -> Vec<String> {
    let mut argv = vec!["dam", "new", "x"];
    argv.extend_from_slice(flags);
    match parsed(&argv).command {
        Command::New(a) => a.labels,
        _ => panic!("wrong command"),
    }
}

fn refusal(argv: &[&str]) -> String {
    let err = Cli::try_parse_from(argv).unwrap_err();
    assert_eq!(err.exit_code(), 2, "{argv:?}");
    err.to_string()
}

#[test]
fn a_comma_list_gives_one_label_per_piece_on_new() {
    assert_eq!(added_on_new(&["--labels", "small,big"]), ["small", "big"]);
}

#[test]
fn space_around_a_piece_is_trimmed() {
    assert_eq!(added_on_new(&["--labels", "a, b"]), ["a", "b"]);
}

#[test]
fn the_repeated_form_still_works_and_combines_with_a_list() {
    assert_eq!(
        added_on_new(&["--labels", "a,b", "--label", "c"]),
        ["a", "b", "c"]
    );
}

#[test]
fn edit_splits_both_the_labels_it_adds_and_the_labels_it_removes() {
    match parsed(&[
        "dam",
        "edit",
        "abcd",
        "--labels",
        "a,b",
        "--unlabels",
        "c,d",
    ])
    .command
    {
        Command::Edit(a) => {
            assert_eq!(a.labels, ["a", "b"]);
            assert_eq!(a.unlabels, ["c", "d"]);
        }
        _ => panic!("wrong command"),
    }
}

#[test]
fn token_create_splits_its_labels() {
    match parsed(&[
        "dam",
        "token",
        "create",
        "bot",
        "--allow",
        "task:read",
        "--labels",
        "a,b",
    ])
    .command
    {
        Command::Token(TokenArgs {
            command: TokenCommand::Create(a),
        }) => assert_eq!(a.labels, ["a", "b"]),
        _ => panic!("wrong command"),
    }
}

#[test]
fn an_empty_piece_is_a_usage_error_that_names_the_flag() {
    for (argv, flag) in [
        (&["dam", "new", "x", "--labels", "a,,b"][..], "--labels"),
        (&["dam", "new", "x", "--labels", "a,"][..], "--labels"),
        (&["dam", "new", "x", "--labels", ""][..], "--labels"),
        (
            &["dam", "edit", "abcd", "--unlabels", "a, ,b"][..],
            "--unlabels",
        ),
        (
            &[
                "dam",
                "token",
                "create",
                "bot",
                "--allow",
                "task:read",
                "--labels",
                ",a",
            ][..],
            "--labels",
        ),
    ] {
        let said = refusal(argv);
        assert!(said.contains(flag), "{said}");
        assert!(said.contains("a label cannot be empty"), "{said}");
    }
}
