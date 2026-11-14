use dam_application::{ConfigFile, RemoteName};

use super::support::{Folder, bare_token};

#[test]
fn the_config_file_port_reads_and_edits_both_files() {
    let folder = Folder::new();
    let port: &dyn ConfigFile = &folder.file();
    let todoist = RemoteName::parse("todoist").unwrap();
    assert_eq!(port.add_remote(&todoist, "todoist::").unwrap(), None);
    assert_eq!(port.add_token(&bare_token("tool")).unwrap(), None);
    let config = port.load().unwrap();
    assert_eq!(config.remote("todoist").unwrap().url, "todoist::");
    assert_eq!(config.tokens.names, ["tool"]);
    assert_eq!(port.remove_token("tool").unwrap(), None);
    assert!(port.load().unwrap().tokens.names.is_empty());
}
