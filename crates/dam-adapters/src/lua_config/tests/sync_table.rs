use std::time::Duration;

use dam_application::{Expected, Misread, RemoteSync};

use super::support::{config, in_table};

fn syncing(sync: &str) -> String {
    format!("return {{ remote = {{ todoist = {{ url = 'todoist::', sync = {sync} }} }} }}")
}

fn read(sync: &str) -> RemoteSync {
    let config = config(&syncing(sync)).load().unwrap();
    config.remote("todoist").unwrap().sync.clone()
}

#[test]
fn the_sync_table_reads_an_interval_and_allowed() {
    let sync = read("{ interval = '5m', allowed = false }");
    assert_eq!(sync.interval(), Duration::from_secs(300));
    assert_eq!(sync.interval.as_ref().unwrap().text, "5m");
    assert!(!sync.allowed());
}

#[test]
fn a_remote_without_a_sync_table_gets_the_default_interval_and_is_allowed() {
    let config = config("return { remote = { todoist = { url = 'todoist::' } } }")
        .load()
        .unwrap();
    let sync = &config.remote("todoist").unwrap().sync;
    assert_eq!(*sync, RemoteSync::default());
    assert_eq!(sync.interval(), Duration::from_secs(15 * 60));
    assert!(sync.allowed());
}

#[test]
fn a_bad_interval_or_allowed_is_refused_by_name() {
    let at = in_table("remote.todoist.sync", "config.lua");
    assert_eq!(
        config(&syncing("{ interval = 'soon' }")).refusal(),
        Misread::NotADuration {
            key: "interval".into(),
            text: "soon".into(),
            at: at.clone(),
        }
        .into()
    );
    assert_eq!(
        config(&syncing("{ allowed = 'no' }")).refusal(),
        Misread::WrongType {
            key: "allowed".into(),
            at: at.clone(),
            expected: Expected::TrueOrFalse,
        }
        .into()
    );
    assert_eq!(
        config(&syncing("{ every = '5m' }")).refusal(),
        Misread::UnknownKey {
            key: "every".into(),
            at,
        }
        .into()
    );
}

#[test]
fn an_interval_shorter_than_a_minute_is_refused_naming_the_floor() {
    assert_eq!(
        config(&syncing("{ interval = '30s' }")).refusal(),
        Misread::ShorterThanFloor {
            key: "interval".into(),
            text: "30s".into(),
            floor: "1m",
            at: in_table("remote.todoist.sync", "config.lua"),
        }
        .into()
    );
    assert_eq!(
        read("{ interval = '60s' }").interval(),
        Duration::from_secs(60)
    );
}

#[test]
fn sync_every_and_stale_are_unknown_remote_keys() {
    for key in ["sync_every", "stale"] {
        let text =
            format!("return {{ remote = {{ todoist = {{ url = 'todoist::', {key} = '15m' }} }} }}");
        assert_eq!(
            config(&text).refusal(),
            Misread::UnknownRemoteKey {
                key: key.into(),
                at: in_table("remote.todoist", "config.lua"),
                takes: ["url", "deadline", "sync", "path"]
                    .map(String::from)
                    .to_vec(),
            }
            .into()
        );
    }
}
