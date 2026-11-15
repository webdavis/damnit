use dam_application::ConfigError;

use super::support::{Folder, ONE_REMOTE};

fn load_refusal(secrets: &str) -> (Folder, ConfigError) {
    let folder = Folder::new().with_config(ONE_REMOTE).with_secrets(secrets);
    let refusal = folder.refusal();
    (folder, refusal)
}

#[test]
fn a_load_withholds_every_name_secrets_lua_holds() {
    for secrets in [
        "return { owner = 'Ada' }",
        "return { remote = { gcal = { api_token = 'abc' } } }",
        "return { remote = { todoist = { api_token = 42 } } }",
    ] {
        let (folder, refusal) = load_refusal(secrets);
        assert_eq!(
            refusal,
            ConfigError::StoppedLoading {
                path: folder.path("secrets.lua"),
                line: None,
                reason: None,
            },
            "{secrets}"
        );
    }
}

#[test]
fn a_load_keeps_a_syntax_error_only_up_to_its_first_quote() {
    let (folder, refusal) = load_refusal("local x = 1\ngoto Ada\nreturn {}\n");
    assert_eq!(
        refusal,
        ConfigError::StoppedLoading {
            path: folder.path("secrets.lua"),
            line: Some(4),
            reason: Some("no visible label".into()),
        }
    );
}
