use clap::Parser;

use crate::args::Cli;

fn refusal(argv: &[&str]) -> String {
    let err = Cli::try_parse_from(argv).unwrap_err();
    assert_eq!(err.exit_code(), 2, "{argv:?}");
    err.to_string()
}

#[test]
fn a_path_with_an_empty_segment_is_refused_in_one_plain_line() {
    let said = refusal(&["dam", "new", "x", "--path", "a//b"]);
    assert!(
        said.starts_with(
            "error: invalid value 'a//b' for '--path <PATH>': a path has an empty segment\n"
        ),
        "{said}"
    );
}

#[test]
fn a_relative_segment_is_refused_by_its_name_not_a_rust_one() {
    for argv in [
        vec!["dam", "mv", "abcd", "../x"],
        vec!["dam", "new", "x", "--path", "a/./b"],
    ] {
        let said = refusal(&argv);
        assert!(said.contains("cannot be \".\" or \"..\""), "{said}");
        assert!(!said.contains("Segment"), "{said}");
    }
}

#[test]
fn a_path_is_normalised_at_the_boundary() {
    for (given, stored) in [("inbox", "inbox/"), ("/a/b/", "a/b/"), ("", "")] {
        let given = ["dam", "new", "x", "--path", given];
        let cli = Cli::try_parse_from(given).unwrap();
        let crate::args::Command::New(a) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(a.path.as_str(), stored);
    }
}
