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

fn parsed(line: &str) -> SyncCommand {
    let words = std::iter::once("dam").chain(line.split(' '));
    match Cli::try_parse_from(words).unwrap().command {
        Command::Sync(SyncArgs { command: Some(c) }) => c,
        other => panic!("wrong command: {other:?}"),
    }
}

#[test]
fn start_takes_a_temporary_interval() {
    assert!(matches!(
        parsed("sync start todoist --interval 5m"),
        SyncCommand::Start { remote, interval: Some(i) }
            if remote == "todoist" && i == Duration::from_secs(300)
    ));
    assert!(matches!(
        parsed("sync start todoist"),
        SyncCommand::Start { interval: None, .. }
    ));
}

#[test]
fn enable_and_disable_act_now_only_when_asked() {
    assert!(matches!(
        parsed("sync enable todoist"),
        SyncCommand::Enable { now: false, .. }
    ));
    assert!(matches!(
        parsed("sync enable todoist --now"),
        SyncCommand::Enable { now: true, .. }
    ));
    assert!(matches!(
        parsed("sync disable todoist --now"),
        SyncCommand::Disable { now: true, .. }
    ));
    assert!(matches!(
        parsed("sync stop todoist"),
        SyncCommand::Stop { .. }
    ));
}

#[test]
fn enable_takes_no_interval_and_stop_takes_no_now() {
    let refused = [
        "sync enable todoist --interval 5m",
        "sync stop todoist --now",
    ];
    for line in refused {
        let words = std::iter::once("dam").chain(line.split(' '));
        assert!(Cli::try_parse_from(words).is_err(), "{line}");
    }
}

#[test]
fn each_command_names_exactly_one_remote() {
    for verb in ["start", "stop", "enable", "disable"] {
        assert!(
            Cli::try_parse_from(["dam", "sync", verb]).is_err(),
            "{verb}"
        );
        let two = ["dam", "sync", verb, "a", "b"];
        assert!(Cli::try_parse_from(two).is_err(), "{verb}");
    }
}
