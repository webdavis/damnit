use dam_application::HookFingerprint;

use super::support::Folder;
use crate::ProjectHookFolder;

fn marking(marker: &std::path::Path) -> String {
    format!("io.open({marker:?}, 'w'):close()\nreturn function() end\n")
}

#[test]
fn the_fingerprint_covers_every_file_in_the_folder_by_name_and_bytes() {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    std::fs::write(hooks.join("task_new.lua"), "abc").unwrap();
    std::fs::write(hooks.join("task_done\r.lua"), "hidden").unwrap();
    std::fs::write(hooks.join("notes.txt"), "not a hook").unwrap();
    let read = ProjectHookFolder::read(&folder.path("app")).unwrap();
    let fingerprint = read.fingerprint().unwrap().unwrap();
    let names: Vec<&str> = fingerprint.files().keys().map(String::as_str).collect();
    assert_eq!(
        names,
        [
            ".damhooks/notes.txt",
            ".damhooks/task_done\\u{d}.lua",
            ".damhooks/task_new.lua"
        ]
    );
    assert_eq!(
        fingerprint.files()[".damhooks/task_new.lua"],
        HookFingerprint::of([("x", "abc".as_bytes())]).files()["x"]
    );
}

#[test]
fn the_bytes_evaluated_are_the_bytes_read_even_when_the_file_is_swapped_after() {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let looked_at = folder.path("looked-at");
    let swapped_in = folder.path("swapped-in");
    std::fs::write(hooks.join("task_new.lua"), marking(&looked_at)).unwrap();
    let read = ProjectHookFolder::read(&folder.path("app")).unwrap();
    let fingerprint = read.fingerprint().unwrap();
    std::fs::write(hooks.join("task_new.lua"), marking(&swapped_in)).unwrap();
    folder.file().project_hooks(&read, "Ada").unwrap();
    assert!(looked_at.exists());
    assert!(!swapped_in.exists());
    assert_eq!(read.fingerprint().unwrap(), fingerprint);
}
