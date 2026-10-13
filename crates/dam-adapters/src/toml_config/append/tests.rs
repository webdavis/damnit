use super::append_remote;
use crate::toml_config::load_config;
use std::path::Path as FsPath;

#[test]
fn append_remote_adds_a_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[done]\ninteractive = true").unwrap();
    append_remote(&path, "todoist", "todoist::").unwrap();
    let c = load_config(&path).unwrap();
    assert_eq!(c.remote("todoist").unwrap().helper, "todoist");
    assert!(c.done_interactive);
}

#[test]
fn append_remote_refuses_a_config_it_cannot_read_back_and_leaves_the_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    append_remote(&path, "todoist", "todoist::").unwrap();
    let before = std::fs::read(&path).unwrap();
    let refused = [
        ("a.b", "config: remote.a needs a url"),
        ("x]", "config is not TOML"),
        (
            "\"q\"",
            "config: the new config would not hold remote \"\\\"q\\\"\"",
        ),
        (
            "todoist",
            "config is not TOML: line 4, column 9: duplicate key",
        ),
    ];
    for (name, why) in refused {
        let err = append_remote(&path, name, "t::").unwrap_err().to_string();
        assert!(err.contains(why), "{name}: {err}");
    }
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

fn mode_of(path: &FsPath) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn append_remote_creates_a_private_file_in_a_private_directory() {
    let base = tempfile::tempdir().unwrap();
    let dir = base.path().join("dam");
    let path = dir.join("config.toml");
    assert_eq!(append_remote(&path, "todoist", "todoist::").unwrap(), None);
    assert_eq!(mode_of(&path), 0o600);
    assert_eq!(mode_of(&dir), 0o700);
}

#[test]
fn append_remote_tightens_a_world_readable_config() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[done]\ninteractive = true\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let warning = append_remote(&path, "todoist", "todoist::").unwrap();
    assert_eq!(mode_of(&path), 0o600);
    assert!(load_config(&path).unwrap().done_interactive);
    let said = warning.expect("a widened config is warned about");
    assert!(said.contains("644"), "{said}");
    assert!(
        !said.contains(path.to_str().unwrap()),
        "the warning carries the config path: {said}"
    );
}
