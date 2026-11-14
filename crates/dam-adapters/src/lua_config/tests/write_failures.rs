use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use dam_application::{ConfigError, ConfigFile, RemoteName};

use super::support::{Folder, bare_token};

fn set_mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

fn writes_anyway(folder: &Path) -> bool {
    let probe = folder.join("probe");
    let wrote = std::fs::write(&probe, "").is_ok();
    let _ = std::fs::remove_file(&probe);
    wrote
}

#[test]
fn a_failed_write_names_the_file_asked_for_and_says_writing() {
    let folder = Folder::new()
        .with_config("return {}\n")
        .with_secrets("return {}\n");
    let dir = folder.path("");
    set_mode(&dir, 0o500);
    if writes_anyway(&dir) {
        set_mode(&dir, 0o700);
        eprintln!("skipped: this user writes into a folder marked read-only");
        return;
    }
    let file = folder.file();
    let todoist = RemoteName::parse("todoist").unwrap();
    let refusals = [
        ("config.lua", file.add_remote(&todoist, "todoist::")),
        ("secrets.lua", file.add_token(&bare_token("tool"))),
    ];
    set_mode(&dir, 0o700);
    for (name, refusal) in refusals {
        let ConfigError::Unwritable { path, why } = refusal.unwrap_err() else {
            panic!("{name} failed some other way");
        };
        assert_eq!(path, folder.path(name));
        assert!(why.to_lowercase().contains("permission denied"), "{why}");
        assert!(!why.contains(".tmp"), "{why}");
    }
}
