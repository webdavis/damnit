use std::time::Duration;

use dam_application::Misread;

use super::support::{config, in_table};

fn syncing_every(every: &str) -> String {
    format!("return {{ remote = {{ todoist = {{ url = 'todoist::', sync_every = '{every}' }} }} }}")
}

#[test]
fn sync_every_reads_as_a_duration() {
    let config = config(&syncing_every("15m")).load().unwrap();
    let every = config
        .remote("todoist")
        .unwrap()
        .sync_every
        .clone()
        .unwrap();
    assert_eq!(
        (every.text.as_str(), every.value),
        ("15m", Duration::from_secs(900))
    );
}

#[test]
fn a_remote_without_sync_every_syncs_only_when_asked() {
    let config = config("return { remote = { todoist = { url = 'todoist::' } } }")
        .load()
        .unwrap();
    assert_eq!(config.remote("todoist").unwrap().sync_every, None);
}

#[test]
fn a_sync_every_of_one_minute_is_kept() {
    let config = config(&syncing_every("60s")).load().unwrap();
    assert!(config.remote("todoist").unwrap().sync_every.is_some());
}

#[test]
fn a_sync_every_under_one_minute_is_refused_naming_the_key_and_the_limit() {
    assert_eq!(
        config(&syncing_every("59s")).refusal(),
        Misread::DurationTooShort {
            key: "sync_every".into(),
            text: "59s".into(),
            shortest: "1m".into(),
            at: in_table("remote.todoist", "config.lua"),
        }
        .into()
    );
}
