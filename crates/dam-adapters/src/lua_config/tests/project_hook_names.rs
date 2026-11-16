use dam_application::{ConfigError, HookEvent};

use super::support::Folder;

const ESCAPE: &str = "task_new\u{1b}[2J.lua";
const CARRIAGE_RETURN: &str = "task_done\r.lua";

fn project_holding(names: &[&str]) -> Folder {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let marker = folder.marker();
    for name in names {
        let hook = format!("io.open({marker:?}, 'w'):close()\nreturn function() end\n");
        std::fs::write(hooks.join(name), hook).unwrap();
    }
    folder
}

#[test]
fn a_name_holding_a_control_character_is_counted_never_named() {
    let folder = project_holding(&[ESCAPE, CARRIAGE_RETURN, "task_changed.lua"]);
    let (names, unshown) = crate::LuaConfigFile::project_hook_names(&folder.path("app")).unwrap();
    assert_eq!(
        names,
        [(
            ".damhooks/task_changed.lua".to_string(),
            Some(HookEvent::TaskChanged)
        )]
    );
    assert_eq!(unshown, 2);
}

#[test]
fn a_file_whose_name_holds_a_control_character_is_never_loaded_or_listed() {
    let folder = project_holding(&[ESCAPE, CARRIAGE_RETURN]);
    let file = folder.file();
    let loaded = file.project_hooks(&folder.path("app"), "Ada").unwrap();
    assert!(loaded.hooks.is_empty());
    let (files, unshown) = file.project_hook_files(&folder.path("app")).unwrap();
    assert!(files.is_empty());
    assert_eq!(unshown, 2);
    assert!(!folder.marker().exists());
}

#[test]
fn a_projects_hook_that_does_not_load_never_quotes_what_it_raised() {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    std::fs::write(hooks.join("task_new.lua"), "error('\\27[2J gone')\n").unwrap();
    let (files, _) = folder
        .file()
        .project_hook_files(&folder.path("app"))
        .unwrap();
    assert_eq!(
        files[0].read,
        Err(ConfigError::StoppedLoading {
            path: hooks.join("task_new.lua"),
            line: None,
            reason: None,
        })
    );
}
