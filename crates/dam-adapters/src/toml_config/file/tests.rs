use super::TomlConfigFile;
use dam_application::{ConfigFile, RemoteName};

#[test]
fn an_added_remote_is_listed_with_its_url_when_the_file_loads_again() {
    let dir = tempfile::tempdir().unwrap();
    let file = TomlConfigFile::new(dir.path().join("config.toml"));
    let warning = file
        .add_remote(&RemoteName("todoist".into()), "todoist::default")
        .unwrap();
    assert_eq!(warning, None);
    let config = file.load().unwrap();
    let remote = config.remote("todoist").unwrap();
    assert_eq!(remote.helper, "todoist");
    assert_eq!(remote.url, "todoist::default");
}
