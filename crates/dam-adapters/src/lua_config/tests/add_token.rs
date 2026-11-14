use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;

use dam_application::{ConfigFile, NewToken, Standing};
use dam_domain::{Channel, Grant, Path, Permission};

use super::support::{Folder, bare_token, secrets};

fn full_token() -> NewToken {
    NewToken {
        expires: Some("2999-05-11T00:00:00Z".parse().unwrap()),
        via: Some(BTreeSet::from([Channel::Local, Channel::Mcp])),
        require: vec!["session".into()],
        grant: Grant {
            allow: BTreeSet::from([Permission::TaskRead]),
            paths: Some(vec![Path::parse("work").unwrap()]),
            labels: Some(BTreeSet::from(["home".to_string()])),
            own: true,
            remotes: Some(vec!["todoist".into()]),
        },
        ..bare_token("tool")
    }
}

fn mode_of(path: &std::path::Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn a_missing_secrets_lua_is_written_private_with_the_schema_and_the_token() {
    let folder = Folder::new();
    let token = full_token();
    assert_eq!(folder.file().add_token(&token).unwrap(), None);
    let value = token.value.expose();
    assert_eq!(
        folder.text("secrets.lua"),
        format!(
            "return {{\n  schema = 1,\n  token = {{\n    tool = {{\n      value = \"{value}\",\n      \
             expires = \"2999-05-11T00:00:00Z\",\n      via = {{ \"local\", \"mcp\" }},\n      \
             require = {{ \"session\" }},\n      grant = {{\n        {{ allow = {{ \"task:read\" }}, \
             paths = {{ \"work/\" }}, labels = {{ \"home\" }}, own = true, remotes = {{ \"todoist\" }} }},\n      \
             }},\n    }},\n  }},\n}}\n"
        )
    );
    assert_eq!(mode_of(&folder.path("secrets.lua")), 0o600);
}

#[test]
fn the_new_token_reads_back_with_every_term() {
    let folder = Folder::new();
    let token = full_token();
    folder.file().add_token(&token).unwrap();
    let config = folder.load().unwrap();
    let entry = &config.tokens.entries[0];
    assert_eq!(entry.name, "tool");
    assert_eq!(entry.value, token.value);
    let Standing::Usable(terms) = &entry.standing else {
        panic!("{entry:?}");
    };
    assert_eq!(terms.expires, token.expires);
    assert_eq!(Some(&terms.via), token.via.as_ref());
    assert_eq!(terms.require, token.require);
    assert_eq!(terms.grants, vec![token.grant]);
}

#[test]
fn the_token_goes_last_in_the_token_table_and_every_comment_stays() {
    let before = "-- dam reads this file\nreturn {\n  schema = 1,\n  remote = {\n    \
                  todoist = { api_token = \"abc\" }, -- the sync key\n  },\n  token = {\n    \
                  -- the tool on my laptop\n    laptop = { value = \"x\" },\n  },\n}\n";
    let folder = secrets(before);
    let token = bare_token("tool");
    folder.file().add_token(&token).unwrap();
    let value = token.value.expose();
    assert_eq!(
        folder.text("secrets.lua"),
        before.replace(
            "    laptop = { value = \"x\" },\n",
            &format!(
                "    laptop = {{ value = \"x\" }},\n    tool = {{\n      value = \"{value}\",\n      \
                 grant = {{\n        {{ allow = {{ \"task:read\" }} }},\n      }},\n    }},\n"
            )
        )
    );
}

#[test]
fn a_token_the_file_already_holds_is_refused_by_name_and_the_file_is_kept() {
    let folder = Folder::new();
    folder.file().add_token(&bare_token("tool")).unwrap();
    let before = folder.text("secrets.lua");
    let why = folder.file().add_token(&full_token()).unwrap_err();
    assert_eq!(
        why.to_string(),
        "config: secrets.lua already holds token tool"
    );
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_table_built_in_code_is_refused_without_the_token_value() {
    for before in [
        "local t = { schema = 1 }\nreturn t\n",
        "local function tokens() return {} end\nreturn { token = tokens() }\n",
    ] {
        let folder = secrets(before);
        let token = full_token();
        let why = folder.file().add_token(&token).unwrap_err().to_string();
        assert!(why.contains("built in code"), "{why}");
        assert!(why.contains("token tool"), "{why}");
        assert!(!why.contains(&token.value.expose()), "{why}");
        assert_eq!(folder.text("secrets.lua"), before);
    }
}

#[test]
fn a_secrets_lua_others_could_read_is_written_private_with_a_warning() {
    let folder = secrets("return {}\n");
    let path = folder.path("secrets.lua");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let said = folder.file().add_token(&bare_token("tool")).unwrap();
    let said = said.expect("a widened secrets.lua is warned about");
    assert!(
        said.contains("644") && said.contains("secrets.lua"),
        "{said}"
    );
    assert!(!said.contains(path.to_str().unwrap()), "{said}");
    assert_eq!(mode_of(&path), 0o600);
}
