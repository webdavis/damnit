use super::*;
use crate::ports::RemoteName;
use crate::testing::todoist;

const TOKEN: &str = "SUPERSECRETTOKEN";

#[test]
fn a_config_holding_a_literal_credential_never_debug_prints_it() {
    let config = Config {
        remotes: vec![RemoteConfig {
            name: todoist(),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: TOKEN.into(),
            }],
            deadline: None,
            sync: Default::default(),
            path: None,
            except: Vec::new(),
        }],
        ..Config::default()
    };
    assert!(!format!("{config:?}").contains(TOKEN));
}

#[test]
fn the_address_is_the_text_after_the_helper_name() {
    let mut remote = RemoteConfig {
        name: RemoteName("gcal".into()),
        helper: "gcal".into(),
        url: "gcal::primary,team@group.calendar.google.com".into(),
        credentials: vec![],
        deadline: None,
        sync: Default::default(),
        path: None,
        except: Vec::new(),
    };
    assert_eq!(remote.address(), "primary,team@group.calendar.google.com");
    remote.url = "todoist::".into();
    assert_eq!(remote.address(), "");
}

fn seconds(text: &str) -> Option<u64> {
    ConfiguredDuration::parse(text).map(|d| d.value.as_secs())
}

#[test]
fn a_duration_is_seconds_minutes_or_hours_and_keeps_its_text() {
    assert_eq!(seconds("30s"), Some(30));
    assert_eq!(seconds("15m"), Some(900));
    assert_eq!(seconds("2h"), Some(7200));
    assert_eq!(ConfiguredDuration::parse("2h").unwrap().text, "2h");
    assert_eq!(seconds("15"), None);
    assert_eq!(seconds("1d"), None);
    assert_eq!(seconds("+15m"), None);
}

#[test]
fn a_non_ascii_empty_or_overflowing_duration_is_refused_rather_than_a_panic() {
    assert_eq!(seconds("15\u{00b5}"), None);
    assert_eq!(seconds(""), None);
    assert_eq!(seconds("\u{201c}2m\u{201d}"), None);
    assert_eq!(seconds("18446744073709551615h"), None);
}

#[test]
fn project_hooks_are_off_until_a_clone_or_the_person_turns_them_on() {
    let personal = |on| Config {
        project_hooks: on,
        ..Config::default()
    };
    assert!(!personal(None).project_hooks_on(None));
    assert!(personal(Some(true)).project_hooks_on(None));
    assert!(personal(None).project_hooks_on(Some(true)));
    assert!(!personal(Some(true)).project_hooks_on(Some(false)));
    assert!(personal(Some(false)).project_hooks_on(Some(true)));
}
