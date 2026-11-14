use dam_application::{ByHand, ConfigError, ConfigFile, EditedTable, TokenName, Uneditable};

use super::support::{Folder, held_already, secrets};

fn tool() -> TokenName {
    TokenName::parse("tool").unwrap()
}

fn refusal(folder: &Folder) -> ConfigError {
    let before = folder.text("secrets.lua");
    let why = folder.file().can_add_token(&tool()).unwrap_err();
    assert_eq!(folder.text("secrets.lua"), before);
    why
}

#[test]
fn a_secrets_lua_written_out_in_full_can_take_the_token_and_is_left_as_it_was() {
    let folder = secrets("return {\n  schema = 1,\n  token = {},\n}\n");
    folder.file().can_add_token(&tool()).unwrap();
    assert_eq!(
        folder.text("secrets.lua"),
        "return {\n  schema = 1,\n  token = {},\n}\n"
    );
}

#[test]
fn without_secrets_lua_the_token_can_be_added_and_no_file_is_made() {
    let folder = Folder::new();
    folder.file().can_add_token(&tool()).unwrap();
    assert!(!folder.path("secrets.lua").exists());
}

#[test]
fn a_token_table_built_in_code_cannot_take_the_token() {
    let folder = secrets("local tokens = {}\nreturn { token = tokens }\n");
    assert_eq!(
        refusal(&folder),
        ConfigError::CannotEdit {
            table: EditedTable::Tokens,
            why: Uneditable::TableBuiltInCode,
            by_hand: ByHand::AddToken {
                name: "tool".into()
            },
        }
    );
}

#[test]
fn a_name_the_file_holds_cannot_be_added_again() {
    let folder = secrets("return { token = { tool = { value = 'a' }, tool = { value = 'b' } } }\n");
    assert_eq!(refusal(&folder), held_already(EditedTable::Tokens, "tool"));
}

#[test]
fn a_secrets_lua_that_does_not_load_cannot_take_the_token() {
    let folder = secrets("return { token = {\n");
    let refusal = refusal(&folder);
    assert!(
        matches!(&refusal, ConfigError::StoppedLoading { path, .. } if path == &folder.path("secrets.lua")),
        "{refusal:?}"
    );
}
