mod support;

use serde_json::Value;
use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn answer(out: &std::process::Output, schema: &str) -> Value {
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, schema);
    answer
}

fn files(answer: &Value) -> Vec<String> {
    let files = answer["files"].as_array().unwrap().iter();
    files.map(|f| f.as_str().unwrap().to_string()).collect()
}

#[test]
fn sync_alone_lists_enable_and_disable() {
    let _guard = support::guard("sync_alone_lists_enable_and_disable");
    let (ok, out, err) = Sandbox::new().dam(&["sync"]);
    assert!(ok, "{err}");
    assert!(out.contains("\n  enable   Have the system run"), "{out}");
    assert!(
        out.contains("\n  disable  Stop the system running"),
        "{out}"
    );
}

#[test]
fn enable_installs_an_entry_running_this_dam_on_these_files_and_disable_removes_it() {
    let _guard = support::guard("enable_installs_an_entry_running_this_dam_on_these_files");
    let sb = Sandbox::new();

    let enabled = answer(
        &sb.scheduling(&["sync", "enable", "--json"]),
        "sync-enable.schema.json",
    );
    assert_eq!(enabled["schedule"], "installed");
    let last_run = sb.dir.path().join("state/dam/sync-last-run.txt");
    assert_eq!(enabled["last_run"], last_run.display().to_string());
    let written = files(&enabled);
    assert!(!written.is_empty());
    for file in &written {
        assert!(
            file.starts_with(&sb.dir.path().display().to_string()),
            "{file}"
        );
    }
    let entry: String = written
        .iter()
        .map(|f| std::fs::read_to_string(f).unwrap())
        .collect();
    let config = sb.config_path().display().to_string();
    let store = sb.dir.path().join("dam.db").display().to_string();
    let state = sb.dir.path().join("state").display().to_string();
    let named = [env!("CARGO_BIN_EXE_dam"), &config, &store, &state];
    for named in named {
        assert!(entry.contains(named), "{named} is missing from {entry}");
    }

    let again = answer(
        &sb.scheduling(&["sync", "enable", "--json"]),
        "sync-enable.schema.json",
    );
    assert_eq!(again["schedule"], "already_installed");
    assert_eq!(files(&again), written);

    let disabled = answer(
        &sb.scheduling(&["sync", "disable", "--json"]),
        "sync-disable.schema.json",
    );
    assert_eq!(disabled["schedule"], "removed");
    assert_eq!(files(&disabled), written);
    assert!(written.iter().all(|f| !std::path::Path::new(f).exists()));

    let again = answer(
        &sb.scheduling(&["sync", "disable", "--json"]),
        "sync-disable.schema.json",
    );
    assert_eq!(
        again,
        serde_json::json!({"schedule": "not_installed", "files": []})
    );
    let calls = sb.scheduler_calls();
    assert!(!calls.is_empty());
    assert!(
        calls
            .iter()
            .all(|c| c.starts_with("launchctl ") || c.starts_with("systemctl --user "))
    );
}

#[test]
fn enable_says_which_remotes_sync_and_when_none_does() {
    let _guard = support::guard("enable_says_which_remotes_sync_and_when_none_does");
    let sb = Sandbox::new();
    let out = sb.scheduling(&["sync", "enable"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{out:?}");
    assert!(text.starts_with("✓ schedule installed: dam sync run starts every minute\n  wrote "));
    assert!(text.ends_with("⚠ no remote has sync_every; set it on a remote in config.lua\n"));

    sb.add_remote("fake", Remote::fake().sync_every("15m"));
    let out = sb.scheduling(&["sync", "enable"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with("· schedule already installed: dam sync run starts every minute\n  kept ")
    );
    assert!(text.ends_with("· fake syncs every 15m\n"), "{text}");

    let out = sb.scheduling(&["sync", "disable"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with("✓ schedule removed\n  removed "), "{text}");
    let out = sb.scheduling(&["sync", "disable"]);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "· schedule not installed\n"
    );
}

#[test]
fn a_token_is_refused_enable_and_disable_and_nothing_is_scheduled() {
    let _guard = support::guard("a_token_is_refused_enable_and_disable");
    let sb = Sandbox::new();
    for verb in ["enable", "disable"] {
        let out = sb.scheduling_holding(Some("damk_v1_whatever"), &["sync", verb, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{out:?}");
        let document = support::errors::error_document(&out);
        assert_eq!(document["error"]["rule"], "not_open_to_tokens");
    }
    assert_eq!(sb.scheduler_calls(), Vec::<String>::new());
}

#[cfg(target_os = "macos")]
#[test]
fn on_macos_enable_bootstraps_a_launch_agent_and_disable_boots_it_out() {
    use std::os::unix::fs::MetadataExt;
    let _guard = support::guard("on_macos_enable_bootstraps_a_launch_agent");
    let sb = Sandbox::new();
    let plist = sb.dir.path().join("Library/LaunchAgents/dam-sync.plist");
    let domain = format!("gui/{}", std::fs::metadata(sb.dir.path()).unwrap().uid());
    let service = format!("{domain}/dam-sync");

    let enabled = answer(
        &sb.scheduling(&["sync", "enable", "--json"]),
        "sync-enable.schema.json",
    );
    assert_eq!(files(&enabled), [plist.display().to_string()]);
    sb.scheduling(&["sync", "disable"]);
    assert_eq!(
        sb.scheduler_calls(),
        [
            format!("launchctl print {service}"),
            format!("launchctl bootstrap {domain} {}", plist.display()),
            format!("launchctl print {service}"),
            format!("launchctl bootout {service}"),
        ]
    );
}
