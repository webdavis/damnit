use dam_application::{ConfigFile, RemoteName};

use super::support::{Folder, bare_token};

#[test]
fn any_url_reads_back_as_written() {
    for url in [
        "todoist::",
        "todoist::say \"hi\"",
        "todoist::back\\slash",
        "todoist::two\nlines\r\n",
        "todoist::tab\there",
        "todoist::bell\u{7}9 and nul\u{0}1",
        "todoist::delete\u{7f}",
        "todoist::ünïcode ✓",
        "todoist::]] and [[",
    ] {
        let folder = Folder::new();
        folder
            .file()
            .add_remote(&RemoteName::parse("todoist").unwrap(), url)
            .unwrap();
        assert_eq!(folder.load().unwrap().remote("todoist").unwrap().url, url);
    }
}

#[test]
fn any_token_name_reads_back_as_written() {
    for name in ["tool", "_tool_2", "my-tool", "2fa", "end", "local"] {
        let folder = Folder::new();
        folder.file().add_token(&bare_token(name)).unwrap();
        assert_eq!(folder.load().unwrap().tokens.names, [name]);
    }
    let folder = Folder::new();
    folder.file().add_token(&bare_token("my-tool")).unwrap();
    assert!(folder.text("secrets.lua").contains("[\"my-tool\"] = {"));
}
