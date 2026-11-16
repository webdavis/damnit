use std::path::Path;

use dam_application::ConfigError;

use super::support::Folder;
use crate::ProjectHookFolder;

fn marking(marker: &Path) -> String {
    format!("io.open({marker:?}, 'w'):close()\nreturn {{}}\n")
}

fn project(task_new: &str) -> Folder {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(hooks.join("lib")).unwrap();
    std::fs::write(hooks.join("task_new.lua"), task_new).unwrap();
    folder
}

fn loaded(folder: &Folder) -> Result<(), ConfigError> {
    let read = ProjectHookFolder::read(&folder.path("app")).unwrap();
    folder.file().project_hooks(&read, "Ada").map(|_| ())
}

#[test]
fn a_hook_requires_a_module_from_its_folder_and_subfolders() {
    let folder = project("require('lib.helpers')\nrequire('lib.util')\nreturn function() end");
    let hooks = folder.path("app").join(".damhooks");
    std::fs::write(
        hooks.join("lib/helpers.lua"),
        marking(&folder.path("helpers")),
    )
    .unwrap();
    std::fs::write(hooks.join("lib/util.lua"), marking(&folder.path("util"))).unwrap();
    loaded(&folder).unwrap();
    assert!(folder.path("helpers").exists());
    assert!(folder.path("util").exists());
}

#[test]
fn a_required_module_outside_the_folder_is_never_found() {
    let folder = project("require('outside')\nreturn function() end");
    std::fs::write(folder.path("app/outside.lua"), marking(&folder.marker())).unwrap();
    std::fs::write(folder.path("outside.lua"), marking(&folder.marker())).unwrap();
    assert!(loaded(&folder).is_err());
    assert!(!folder.marker().exists());
}

#[test]
fn a_required_module_comes_from_the_bytes_checked_even_when_swapped_after() {
    let folder = project("require('lib.helpers')\nreturn function() end");
    let helpers = folder.path("app/.damhooks/lib/helpers.lua");
    std::fs::write(&helpers, marking(&folder.path("looked-at"))).unwrap();
    let read = ProjectHookFolder::read(&folder.path("app")).unwrap();
    std::fs::write(&helpers, marking(&folder.path("swapped-in"))).unwrap();
    folder.file().project_hooks(&read, "Ada").unwrap();
    assert!(folder.path("looked-at").exists());
    assert!(!folder.path("swapped-in").exists());
}
