mod support;

use support::sandbox::Sandbox;

/// What a program reading busy times relies on, read strictly: the one key,
/// integer seconds, a boolean, and nothing else required.
fn busy_intervals(document: &str) -> Vec<(u64, u64, bool)> {
    let value: serde_json::Value = serde_json::from_str(document).unwrap();
    value["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["start"].as_u64().unwrap(),
                e["end"].as_u64().unwrap(),
                e["busy"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[test]
fn dam_agenda_json_answers_every_event_in_the_window_as_intervals() {
    let _guard = support::guard("dam_agenda_json_answers_every_event_in_the_window_as_intervals");
    let sb = Sandbox::new();
    sb.new_object(&[
        "--event",
        "standup",
        "--start",
        "2026-09-25T14:00",
        "--end",
        "2026-09-25T14:30",
    ]);
    sb.new_object(&[
        "--event",
        "offsite",
        "--start",
        "2026-09-25",
        "--end",
        "2026-09-26",
    ]);
    sb.new_object(&[
        "--event",
        "next week",
        "--start",
        "2026-10-02T09:00",
        "--end",
        "2026-10-02T10:00",
    ]);
    let (ok, out, err) = sb.dam(&[
        "agenda",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-26",
        "--json",
    ]);
    assert!(ok, "{err}");
    assert_eq!(
        busy_intervals(&out),
        vec![
            (1_790_294_400, 1_790_380_800, true),
            (1_790_344_800, 1_790_346_600, true)
        ]
    );
}

/// A reader on a deadline runs `agenda --no-pull`: however stale a remote
/// is, no credential command and no helper runs, and those are the only ways
/// dam reaches a vault or the network.
#[test]
fn agenda_no_pull_runs_no_credential_command_and_no_helper() {
    let _guard = support::guard("agenda_no_pull_runs_no_credential_command_and_no_helper");
    let sb = Sandbox::new();
    let resolved = sb.dir.path().join("credential-command-ran");
    // The fake helper writes this file as soon as it starts.
    let launched = sb.dir.path().join("token.txt");
    std::fs::write(
        sb.dir.path().join("config.toml"),
        format!(
            "[remote.fake]\nurl = \"fake::\"\napi_token_command = [\"sh\", \"-c\", \"touch '{}'; echo tok-123\"]\nstale = \"1s\"\n",
            resolved.display()
        ),
    )
    .unwrap();

    // The window is read first, so a flag dam cannot read spawns nothing.
    let mistyped = sb.output(&["agenda", "--from", "someday", "--json"]);
    assert_eq!(mistyped.status.code(), Some(2));
    assert!(
        !resolved.exists(),
        "a mistyped --from ran a credential command"
    );

    let (ok, out, err) = sb.dam(&["agenda", "--no-pull", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![]);
    assert!(!resolved.exists(), "--no-pull ran a credential command");
    assert!(!launched.exists(), "--no-pull launched a helper");

    let (ok, _, err) = sb.dam(&["agenda", "--json"]);
    assert!(ok, "{err}");
    assert!(
        resolved.exists(),
        "without the flag the stale remote's credential command runs"
    );
    assert!(
        launched.exists(),
        "without the flag the stale remote is pulled"
    );
}
