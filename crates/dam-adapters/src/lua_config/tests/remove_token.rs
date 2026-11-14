use std::collections::BTreeSet;

use dam_application::{
    ByHand, ConfigError, ConfigFile, EditedTable, NewToken, TokenName, Uneditable,
};
use dam_domain::{Grant, Permission, Token};

use super::support::{Folder, secrets};

fn removed(before: &str, name: &str) -> String {
    let folder = secrets(before);
    folder.file().remove_token(name).unwrap();
    let names = folder.load().unwrap().tokens.names;
    assert!(!names.iter().any(|n| n == name), "{names:?}");
    folder.text("secrets.lua")
}

fn new_token(name: &str) -> NewToken {
    NewToken {
        name: TokenName::parse(name).unwrap(),
        value: Token::generate(&mut |b: &mut [u8]| b.fill(9)),
        expires: Some("2999-05-11T00:00:00Z".parse().unwrap()),
        via: None,
        require: vec!["session".into()],
        grant: Grant {
            allow: BTreeSet::from([Permission::TaskRead, Permission::TaskCreate]),
            paths: None,
            labels: None,
            own: true,
            remotes: None,
        },
    }
}

#[test]
fn the_token_goes_with_its_lines_and_the_comment_above_it_stays() {
    let before = "return {\n  token = {\n    -- the laptop\n    laptop = { value = \"x\" },\n    \
                  -- the tool\n    tool = {\n      value = \"y\",\n    }, -- made by dam\n  },\n}\n";
    assert_eq!(
        removed(before, "tool"),
        "return {\n  token = {\n    -- the laptop\n    laptop = { value = \"x\" },\n    \
         -- the tool\n  },\n}\n"
    );
}

#[test]
fn adding_then_removing_a_token_keeps_a_table_written_by_hand_byte_for_byte() {
    for before in [
        "-- mine\nreturn {\n  schema = 1,\n  token = {\n    laptop = { value = \"x\" }, -- keep\n  },\n}\n",
        "return {\n  token = {\n    laptop = { value = \"x\" } -- keep\n  }\n}\n",
        "return {\n  token = {\n  },\n}\n",
        "return { token = { laptop = { value = \"x\" } } }\n",
        "return { token = { laptop = { value = \"x\" }, } }\n",
    ] {
        let folder = secrets(before);
        folder.file().add_token(&new_token("tool")).unwrap();
        assert_ne!(folder.text("secrets.lua"), before);
        folder.file().remove_token("tool").unwrap();
        assert_eq!(folder.text("secrets.lua"), before);
    }
}

#[test]
fn a_token_inside_a_one_line_table_goes_with_its_comma() {
    assert_eq!(
        removed(
            "return { token = { a = { value = \"x\" }, tool = { value = \"y\" }, b = { value = \"z\" } } }\n",
            "tool"
        ),
        "return { token = { a = { value = \"x\" }, b = { value = \"z\" } } }\n"
    );
}

#[test]
fn a_token_under_a_quoted_key_goes() {
    assert_eq!(
        removed(
            "return {\n  token = {\n    [\"my-tool\"] = { value = \"y\" },\n  },\n}\n",
            "my-tool"
        ),
        "return {\n  token = {\n  },\n}\n"
    );
}

#[test]
fn a_token_the_file_does_not_hold_is_refused_by_name_and_the_file_is_kept() {
    let before = "return { token = { laptop = { value = \"x\" } } }\n";
    let folder = secrets(before);
    let why = folder.file().remove_token("tool").unwrap_err();
    assert_eq!(why, not_held());
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn without_secrets_lua_there_is_no_token_to_remove_and_no_file_is_made() {
    let folder = Folder::new();
    let why = folder.file().remove_token("tool").unwrap_err();
    assert_eq!(why, not_held());
    assert!(!folder.path("secrets.lua").exists());
}

#[test]
fn a_token_table_built_in_code_is_refused_and_the_file_is_kept() {
    let before = "local function tokens() return {} end\nreturn { token = tokens() }\n";
    let folder = secrets(before);
    assert_eq!(
        folder.file().remove_token("tool").unwrap_err(),
        ConfigError::CannotEdit {
            table: EditedTable::Tokens,
            why: Uneditable::TableBuiltInCode,
            by_hand: ByHand::RemoveToken {
                name: "tool".into()
            },
        }
    );
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_block_comment_beside_the_token_stays_whole() {
    let kept = "--[[ the tool\n     on my laptop ]]";
    for (before, after) in [
        (
            format!(
                "return {{\n  token = {{\n    {kept}\n    tool = {{ value = \"y\" }},\n  }},\n}}\n"
            ),
            format!("return {{\n  token = {{\n    {kept}\n  }},\n}}\n"),
        ),
        (
            format!("return {{\n  token = {{\n    tool = {{ value = \"y\" }}, {kept}\n  }},\n}}\n"),
            format!("return {{\n  token = {{\n    {kept}\n  }},\n}}\n"),
        ),
        (
            format!("return {{\n  token = {{\n    {kept} tool = {{ value = \"y\" }},\n  }},\n}}\n"),
            format!("return {{\n  token = {{\n    {kept}\n  }},\n}}\n"),
        ),
    ] {
        assert_eq!(removed(&before, "tool"), after);
    }
}

#[test]
fn a_token_after_another_on_its_line_goes_with_the_space_before_it() {
    for (before, after) in [
        (
            "return {\n  token = {\n    a = { value = \"x\" }, tool = { value = \"y\" },\n  },\n}\n",
            "return {\n  token = {\n    a = { value = \"x\" },\n  },\n}\n",
        ),
        (
            "return {\n  token = {\n    a = { value = \"x\" }, tool = { value = \"y\" }\n  }\n}\n",
            "return {\n  token = {\n    a = { value = \"x\" }\n  }\n}\n",
        ),
    ] {
        assert_eq!(removed(before, "tool"), after);
    }
}

#[test]
fn a_token_table_the_writer_made_or_emptied_may_stay_changed() {
    for (before, after) in [
        ("return { token = {} }\n", "return { token = {\n} }\n"),
        (
            "return { schema = 1 }\n",
            "return { schema = 1, token = {\n  } }\n",
        ),
        ("return {}\n", "return {\n  token = {\n  },\n}\n"),
    ] {
        let folder = secrets(before);
        folder.file().add_token(&new_token("tool")).unwrap();
        folder.file().remove_token("tool").unwrap();
        assert_eq!(folder.text("secrets.lua"), after);
    }
}

fn not_held() -> ConfigError {
    ConfigError::NotHeld {
        table: EditedTable::Tokens,
        entry: "tool".into(),
    }
}
