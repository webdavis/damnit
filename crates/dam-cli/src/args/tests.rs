//! The parsed command line, asserted through clap itself.

use super::*;
use clap::Parser;
use dam_domain::{ChildDisposition, DependencyDisposition};

#[test]
fn new_with_flags_parses() {
    let cli = Cli::try_parse_from([
        "dam", "new", "buy milk", "--path", "inbox/", "--due", "tomorrow", "-p", "1", "--label",
        "errand",
    ])
    .unwrap();
    match cli.command {
        Command::New(a) => {
            assert_eq!(a.subject, "buy milk");
            assert_eq!(a.path, "inbox/");
            assert_eq!(a.due.as_deref(), Some("tomorrow"));
            assert_eq!(a.priority, Some(1));
            assert_eq!(a.labels, vec!["errand".to_string()]);
        }
        _ => panic!("wrong command"),
    }
}

#[test]
fn json_and_toon_are_global_and_exclusive() {
    let cli = Cli::try_parse_from(["dam", "ls", "--json"]).unwrap();
    assert!(cli.json);
    assert!(Cli::try_parse_from(["dam", "ls", "--json", "--toon"]).is_err());
}

#[test]
fn resolve_needs_exactly_one_side() {
    assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "resolve", "abc"]).is_err());
    assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours", "--theirs"]).is_err());
}

#[test]
fn edit_takes_undone_beside_the_other_clearing_flags() {
    let cli = Cli::try_parse_from(["dam", "edit", "abcd", "--undone"]).unwrap();
    match cli.command {
        Command::Edit(a) => assert!(a.undone),
        _ => panic!("wrong command"),
    }
}

#[test]
fn each_disposition_word_parses_to_its_variant() {
    let done = |args: &[&str]| {
        let mut argv = vec!["dam", "done", "abcd", "--force"];
        argv.extend_from_slice(args);
        match Cli::try_parse_from(argv).unwrap().command {
            Command::Done(a) => a,
            _ => panic!("wrong command"),
        }
    };
    assert_eq!(
        done(&["--children", "up"]).children,
        Some(ChildDisposition::Up)
    );
    assert_eq!(
        done(&["--children", "keep"]).children,
        Some(ChildDisposition::Keep)
    );
    assert_eq!(
        done(&["--children", "into:leftovers"]).children,
        Some(ChildDisposition::Into("leftovers".into()))
    );
    assert_eq!(
        done(&["--depends", "drop"]).depends,
        Some(DependencyDisposition::Drop)
    );
    assert_eq!(
        done(&["--depends", "keep"]).depends,
        Some(DependencyDisposition::Keep)
    );
    assert_eq!(done(&[]).children, None);
    assert_eq!(done(&[]).depends, None);
}

/// clap renders an invalid value as the flag, the value and the accepted
/// set, and points at --help rather than printing a usage line.
#[test]
fn a_disposition_outside_the_set_is_a_usage_error_naming_the_accepted_words() {
    for (bad, accepted) in [
        (vec!["--children", "sideways"], "up, keep, into:<name>"),
        (vec!["--children", "into:"], "up, keep, into:<name>"),
        (vec!["--depends", "maybe"], "drop, keep"),
    ] {
        let mut argv = vec!["dam", "done", "abcd", "--force"];
        argv.extend_from_slice(&bad);
        let err = Cli::try_parse_from(argv).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{bad:?}");
        let rendered = err.to_string();
        assert!(rendered.contains(bad[0]), "{bad:?}: {rendered}");
        assert!(rendered.contains(accepted), "{bad:?}: {rendered}");
        assert!(rendered.contains("--help"), "{bad:?}: {rendered}");
    }
}

/// Ignoring `done.interactive` is what the flags are for; ignoring an
/// --interactive the operator typed on the same line is a different thing.
#[test]
fn a_disposition_beside_an_explicit_interactive_is_refused() {
    for flag in [vec!["--children", "keep"], vec!["--depends", "drop"]] {
        let mut argv = vec!["dam", "done", "abcd", "--force", "--interactive"];
        argv.extend_from_slice(&flag);
        let err = Cli::try_parse_from(argv).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{flag:?}");
        assert!(err.to_string().contains("--interactive"), "{flag:?}: {err}");
    }
}

#[test]
fn a_disposition_without_force_is_refused() {
    assert!(Cli::try_parse_from(["dam", "done", "abcd", "--children", "keep"]).is_err());
    assert!(Cli::try_parse_from(["dam", "done", "abcd", "--depends", "drop"]).is_err());
}

#[test]
fn restore_needs_at_least_one_oid() {
    assert!(Cli::try_parse_from(["dam", "restore", "abcd"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "restore"]).is_err());
}

#[test]
fn add_all_and_oids_are_alternatives() {
    assert!(Cli::try_parse_from(["dam", "add", "-A"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "add", "abc", "def"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "add"]).is_err());
}
