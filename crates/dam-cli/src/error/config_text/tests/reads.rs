use dam_application::{ConfigError, Expected, Misread, Place, RemoteNameError};
use dam_domain::{CategoryError, PathError};

use super::text;

fn top_of(file: &str) -> Place {
    Place {
        file: file.into(),
        table: None,
    }
}

fn in_table(table: &str, file: &str) -> Place {
    Place {
        file: file.into(),
        table: Some(table.into()),
    }
}

#[test]
fn a_file_dam_cannot_read_or_load_is_named_with_its_path() {
    let path = || "/home/ada/.config/dam/config.lua".into();
    assert_eq!(
        text(ConfigError::NotLua("/home/ada/dam.toml".into())),
        "config: /home/ada/dam.toml does not end in .lua; the config is config.lua now"
    );
    assert_eq!(
        text(ConfigError::Unreadable {
            path: path(),
            why: "Permission denied (os error 13)".into(),
        }),
        "reading config: /home/ada/.config/dam/config.lua: Permission denied (os error 13)"
    );
    assert_eq!(
        text(ConfigError::StoppedLoading {
            path: path(),
            line: Some(2),
            reason: Some("unexpected symbol near '='".into()),
        }),
        "config: /home/ada/.config/dam/config.lua:2: unexpected symbol near '='"
    );
    assert_eq!(
        text(ConfigError::StoppedLoading {
            path: path(),
            line: Some(3),
            reason: None,
        }),
        "config: /home/ada/.config/dam/config.lua:3: loading stopped at an error"
    );
    assert_eq!(
        text(ConfigError::StoppedLoading {
            path: path(),
            line: None,
            reason: None,
        }),
        "config: /home/ada/.config/dam/config.lua: loading stopped at an error"
    );
    assert_eq!(
        text(ConfigError::NotATable {
            path: path(),
            returned: "an integer".into(),
        }),
        "config: /home/ada/.config/dam/config.lua must return a table, and it returned an integer"
    );
}

#[test]
fn a_value_dam_cannot_take_is_named_with_its_key_and_table() {
    let cases = [
        (
            ConfigError::NotNamedSettings {
                file: "config.lua".into(),
            },
            "config.lua must return a table of named settings",
        ),
        (
            Misread::UnknownKey {
                key: "colour".into(),
                at: top_of("config.lua"),
            }
            .into(),
            "unknown key colour at the top level of config.lua",
        ),
        (
            Misread::UnknownRemoteKey {
                key: "api_token".into(),
                at: in_table("remote.todoist", "config.lua"),
                takes: vec!["url".into(), "stale".into()],
            }
            .into(),
            "unknown key api_token in remote.todoist of config.lua. A remote takes url, stale; \
             its credentials belong in secrets.lua",
        ),
        (
            Misread::WrongType {
                key: "stale".into(),
                at: in_table("remote.todoist", "config.lua"),
                expected: Expected::Text,
            }
            .into(),
            "stale must be a string in remote.todoist of config.lua",
        ),
        (
            Misread::Missing {
                key: "url".into(),
                at: in_table("remote.todoist", "config.lua"),
            }
            .into(),
            "url is missing in remote.todoist of config.lua",
        ),
        (
            Misread::NotAPath {
                text: "a//b".into(),
                at: in_table("remote.todoist", "config.lua"),
                why: PathError::EmptySegment,
            }
            .into(),
            "path \"a//b\": EmptySegment in remote.todoist of config.lua",
        ),
        (
            Misread::NotAHelperUrl {
                url: "::".into(),
                at: in_table("remote.todoist", "config.lua"),
            }
            .into(),
            "url \"::\" must look like <helper>::, the helper being letters, digits, - and _ \
             in remote.todoist of config.lua",
        ),
        (
            Misread::NotADuration {
                key: "stale".into(),
                text: "+15m".into(),
                at: in_table("remote.todoist", "config.lua"),
            }
            .into(),
            "stale \"+15m\" must be <number><s|m|h> in remote.todoist of config.lua",
        ),
        (
            ConfigError::BelongsInSecrets {
                key: "token".into(),
                at: top_of("config.lua"),
            },
            "token belongs in secrets.lua at the top level of config.lua",
        ),
        (
            ConfigError::BadRemoteName(RemoteNameError::Empty),
            "a remote name cannot be empty",
        ),
        (
            ConfigError::BadCategories(CategoryError::DuplicateName("effort".into())),
            "category: DuplicateName(\"effort\")",
        ),
        (
            ConfigError::UndeclaredRemote {
                remote: "gcal".into(),
                config: "config.lua".into(),
            },
            "remote gcal in secrets.lua is not declared in config.lua",
        ),
    ];
    for (refusal, said) in cases {
        assert_eq!(text(refusal), format!("config: {said}"));
    }
}
