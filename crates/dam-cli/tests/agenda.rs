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
    let mistyped = sb.output(&["agenda", "--max-age", "fake=soon", "--json"]);
    assert_eq!(mistyped.status.code(), Some(2));
    assert!(
        !resolved.exists(),
        "a mistyped --max-age ran a credential command"
    );

    let (ok, out, err) = sb.dam(&["agenda", "--no-pull", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![]);
    assert!(!resolved.exists(), "--no-pull ran a credential command");
    assert!(!launched.exists(), "--no-pull launched a helper");

    // Freshness is judged after the stale pull, so the pull it just made counts.
    let (ok, _, err) = sb.dam(&["agenda", "--json", "--max-age", "fake=1h"]);
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

fn error_document(out: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&out.stderr)
        .unwrap_or_else(|e| panic!("stderr is not one JSON document: {e}"))
}

/// A store nothing has pulled into is refused under `--max-age`, not
/// answered as a clear calendar; once a pull lands, the same read answers.
#[test]
fn max_age_refuses_a_remote_never_pulled_and_answers_once_one_lands() {
    let _guard = support::guard("max_age_refuses_a_remote_never_pulled_and_answers_once_one_lands");
    let sb = Sandbox::new();
    let never = sb.output(&["agenda", "--json", "--no-pull", "--max-age", "fake=1h"]);
    assert_eq!(never.status.code(), Some(4));
    assert!(never.stdout.is_empty(), "a refused read prints no document");
    let refusal = error_document(&never);
    assert_eq!(refusal["error"]["rule"], "stale_remote");
    assert_eq!(
        refusal["error"]["message"],
        "remote \"fake\" has never been pulled; run dam pull fake"
    );

    let (ok, _, err) = sb.dam(&["pull", "fake"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["agenda", "--json", "--no-pull", "--max-age", "fake=1h"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![]);
}

#[test]
fn max_age_takes_a_configured_remote_and_a_duration_it_can_read() {
    let _guard = support::guard("max_age_takes_a_configured_remote_and_a_duration_it_can_read");
    let sb = Sandbox::new();
    let unknown = sb.output(&["agenda", "--json", "--no-pull", "--max-age", "nope=1h"]);
    assert_eq!(unknown.status.code(), Some(4));
    assert_eq!(error_document(&unknown)["error"]["rule"], "no_such_remote");
    for unreadable in ["fake", "fake=", "=1h", "fake=soon", "fake=1d"] {
        let out = sb.output(&["agenda", "--no-pull", "--max-age", unreadable]);
        assert_eq!(out.status.code(), Some(2), "{unreadable}");
    }
}
