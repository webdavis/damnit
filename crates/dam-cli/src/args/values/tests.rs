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

#[test]
fn a_priority_outside_one_to_four_is_refused_with_the_range_in_words() {
    for bad in ["0", "5", "9", "high", ""] {
        let said = refusal(&["dam", "new", "x", "-p", bad]);
        assert!(
            said.contains("priority is 1 (highest) to 4"),
            "{bad:?}: {said}"
        );
        let said = refusal(&["dam", "edit", "abcd", "--priority", bad]);
        assert!(
            said.contains("priority is 1 (highest) to 4"),
            "{bad:?}: {said}"
        );
    }
    for ok in ["1", "2", "3", "4"] {
        assert!(
            Cli::try_parse_from(["dam", "new", "x", "-p", ok]).is_ok(),
            "{ok}"
        );
    }
}

#[test]
fn a_max_age_is_a_remote_name_and_a_duration_or_a_refusal_naming_the_part_at_fault() {
    for (bad, told) in [
        ("fake", "expected <remote>=<number><s|m|h>"),
        ("=1h", "a remote name cannot be empty"),
        ("fa ke=1h", "may hold only letters, digits, - and _"),
        ("fake=", "is not a duration such as 90s, 15m or 2h"),
        ("fake=soon", "is not a duration such as 90s, 15m or 2h"),
        ("fake=1d", "is not a duration such as 90s, 15m or 2h"),
    ] {
        let said = refusal(&["dam", "agenda", "--max-age", bad]);
        assert!(said.contains("--max-age"), "{bad:?}: {said}");
        assert!(said.contains(told), "{bad:?}: {said}");
    }
    assert!(
        Cli::try_parse_from([
            "dam",
            "agenda",
            "--max-age",
            "fake=1h",
            "--max-age",
            "b_2=90s"
        ])
        .is_ok()
    );
}
