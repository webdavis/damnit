use std::os::unix::fs::PermissionsExt;

use dam_application::{ByHand, ConfigError, ConfigFile, EditedTable, RemoteName, TokenName};

use super::support::{Folder, bare_token};

const TOKENS: &str =
    "return {\n  schema = 1,\n  token = {\n    tool = { value = \"x\" },\n  },\n}\n";

fn read_only(folder: &Folder, name: &str) -> String {
    let path = folder.path(name);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();
    folder.text(name)
}

fn mode_of(folder: &Folder, name: &str) -> u32 {
    std::fs::metadata(folder.path(name))
        .unwrap()
        .permissions()
        .mode()
        & 0o777
}

fn token_by_hand(by_hand: ByHand) -> ConfigError {
    ConfigError::ReadOnly {
        table: EditedTable::Tokens,
        by_hand,
    }
}

#[test]
fn a_secrets_lua_marked_read_only_is_left_as_it_is() {
    let folder = Folder::new().with_secrets(TOKENS);
    let before = read_only(&folder, "secrets.lua");
    let file = folder.file();
    let other = || ByHand::AddToken {
        name: "other".into(),
    };
    let refusals = [
        (
            file.can_add_token(&TokenName::parse("other").unwrap())
                .map(|_| None),
            other(),
        ),
        (file.add_token(&bare_token("other")), other()),
        (
            file.remove_token("tool"),
            ByHand::RemoveToken {
                name: "tool".into(),
            },
        ),
    ];
    for (refusal, by_hand) in refusals {
        assert_eq!(refusal.unwrap_err(), token_by_hand(by_hand));
    }
    assert_eq!(folder.text("secrets.lua"), before);
    assert_eq!(mode_of(&folder, "secrets.lua"), 0o400);
}

#[test]
fn a_config_lua_marked_read_only_is_left_as_it_is() {
    let folder = Folder::new().with_config("return {}\n");
    let before = read_only(&folder, "config.lua");
    let todoist = RemoteName::parse("todoist").unwrap();
    assert_eq!(
        folder.file().add_remote(&todoist, "todoist::").unwrap_err(),
        ConfigError::ReadOnly {
            table: EditedTable::Remotes,
            by_hand: ByHand::AddRemote {
                entry: "todoist = { url = \"todoist::\" }".into(),
            },
        }
    );
    assert_eq!(folder.text("config.lua"), before);
    assert_eq!(mode_of(&folder, "config.lua"), 0o400);
}
