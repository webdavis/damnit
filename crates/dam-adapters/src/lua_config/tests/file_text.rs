use dam_application::RemoteName;

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
    let read = folder.refusal();
    assert!(
        read.contains("config.lua") && read.contains("must return a table"),
        "{read}"
    );
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert!(why.to_string().contains("config.lua"), "{why}");
    assert_eq!(folder.text("config.lua"), COMMENTS);
}

#[test]
fn a_secrets_lua_of_only_comments_is_refused_by_the_reader_and_the_writer() {
    let folder = Folder::new().with_secrets(COMMENTS);
    let read = folder.refusal();
    assert!(
        read.contains("secrets.lua") && read.contains("must return a table"),
        "{read}"
    );
    let why = folder.file().add_token(&bare_token("tool")).unwrap_err();
    assert!(why.to_string().contains("secrets.lua"), "{why}");
    assert_eq!(folder.text("secrets.lua"), COMMENTS);
}
