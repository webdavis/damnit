use dam_application::{ConfigError, ConfigFile, RemoteName};

use super::support::{Folder, bare_token};

const COMMENTS: &str = "-- nothing here yet\n";

fn todoist() -> RemoteName {
    RemoteName::parse("todoist").unwrap()
}

fn only_crlf(text: &str) -> bool {
    !text.replace("\r\n", "").contains('\n')
}

#[test]
fn a_config_lua_with_crlf_line_endings_keeps_them() {
    let folder = Folder::new().with_config("return {\r\n  owner = \"Ada\",\r\n}\r\n");
    folder.file().add_remote(&todoist(), "todoist::").unwrap();
    let text = folder.text("config.lua");
    assert!(only_crlf(&text), "{text:?}");
}

#[test]
fn a_secrets_lua_with_crlf_line_endings_keeps_them() {
    let before = "return {\r\n  token = {\r\n    laptop = { value = \"x\" },\r\n  },\r\n}\r\n";
    let folder = Folder::new().with_secrets(before);
    folder.file().add_token(&bare_token("tool")).unwrap();
    let text = folder.text("secrets.lua");
    assert!(only_crlf(&text), "{text:?}");
    folder.file().remove_token("tool").unwrap();
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_blank_config_lua_reads_and_is_written_like_a_missing_one() {
    let missing = Folder::new();
    missing.file().add_remote(&todoist(), "todoist::").unwrap();
    for blank in ["", " \n\t\n"] {
        let folder = Folder::new().with_config(blank);
        assert!(folder.load().is_ok());
        folder.file().add_remote(&todoist(), "todoist::").unwrap();
        assert_eq!(folder.text("config.lua"), missing.text("config.lua"));
    }
}

#[test]
fn a_blank_secrets_lua_reads_and_is_written_like_a_missing_one() {
    let missing = Folder::new();
    missing.file().add_token(&bare_token("tool")).unwrap();
    for blank in ["", " \n\t\n"] {
        let folder = Folder::new().with_secrets(blank);
        assert!(folder.load().is_ok());
        folder.file().add_token(&bare_token("tool")).unwrap();
        assert_eq!(folder.text("secrets.lua"), missing.text("secrets.lua"));
    }
}

#[test]
fn a_config_lua_of_only_comments_is_refused_by_the_reader_and_the_writer() {
    let folder = Folder::new().with_config(COMMENTS);
    let returns_nil = ConfigError::NotATable {
        path: folder.path("config.lua"),
        returned: "nil".into(),
    };
    assert_eq!(folder.refusal(), returns_nil);
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert_eq!(why, returns_nil);
    assert_eq!(folder.text("config.lua"), COMMENTS);
}

#[test]
fn a_secrets_lua_of_only_comments_is_refused_by_the_reader_and_the_writer() {
    let folder = Folder::new().with_secrets(COMMENTS);
    let returns_nil = ConfigError::NotATable {
        path: folder.path("secrets.lua"),
        returned: "nil".into(),
    };
    assert_eq!(folder.refusal(), returns_nil);
    let why = folder.file().add_token(&bare_token("tool")).unwrap_err();
    assert_eq!(why, returns_nil);
    assert_eq!(folder.text("secrets.lua"), COMMENTS);
}
