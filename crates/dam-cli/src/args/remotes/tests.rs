use std::time::Duration;

use clap::Parser;

use crate::args::{Cli, Command, SyncArgs, SyncCommand};

fn run(line: &[&str]) -> (String, bool, Option<Duration>) {
    let cli = Cli::try_parse_from(line).unwrap();
    match cli.command {
        Command::Sync(SyncArgs {
            command:
                Some(SyncCommand::Run {
                    remote,
                    when_due,
                    interval,
                }),
        }) => (remote, when_due, interval),
        other => panic!("wrong command: {other:?}"),
    }
}

#[test]
fn sync_run_syncs_now_unless_asked_to_wait_until_due() {
    assert_eq!(
        run(&["dam", "sync", "run", "todoist"]),
        ("todoist".into(), false, None)
    );
    assert_eq!(
        run(&[
            "dam",
            "sync",
            "run",
            "todoist",
            "--when-due",
            "--interval",
            "5m"
        ]),
        ("todoist".into(), true, Some(Duration::from_secs(300)))
    );
}

#[test]
fn an_interval_shorter_than_a_minute_is_a_usage_error() {
    let err = Cli::try_parse_from([
        "dam",
        "sync",
        "run",
        "todoist",
        "--when-due",
        "--interval",
        "30s",
    ])
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("at least 1m"), "{err}");
}

#[test]
fn an_interval_needs_when_due() {
    assert!(Cli::try_parse_from(["dam", "sync", "run", "todoist", "--interval", "5m"]).is_err());
}
