use std::path::PathBuf;

use dam_application::HookFingerprint;

use super::support::Folder;
use crate::ProjectHookFolder;

struct Hooks(Folder);

fn hooks() -> Hooks {
    let folder = Folder::new();
    std::fs::create_dir_all(folder.path("app").join(".damhooks").join("lib")).unwrap();
    let hooks = Hooks(folder);
    hooks.write("task_new.lua", "return function() end");
    hooks.write("lib/helpers.lua", "return {}");
    hooks
}

impl Hooks {
    fn at(&self, name: &str) -> PathBuf {
        self.0.path("app").join(".damhooks").join(name)
    }

    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.at(name), text).unwrap();
    }

    fn fingerprint(&self) -> Option<HookFingerprint> {
        ProjectHookFolder::read(&self.0.path("app"))
            .unwrap()
            .fingerprint()
            .unwrap()
    }

    fn changes_with(&self, change: impl FnOnce(&Hooks)) -> bool {
        let then = self.fingerprint();
        change(self);
        then != self.fingerprint()
    }
}

#[test]
fn a_changed_helper_file_is_a_change() {
    assert!(hooks().changes_with(|h| h.write("lib/helpers.lua", "return { x = 1 }")));
}

#[test]
fn a_file_added_in_a_subfolder_is_a_change() {
    assert!(hooks().changes_with(|h| h.write("lib/util.lua", "return {}")));
}

#[test]
fn an_empty_subfolder_added_is_a_change() {
    assert!(hooks().changes_with(|h| std::fs::create_dir(h.at("more")).unwrap()));
}

#[test]
fn a_changed_file_whose_name_dam_will_not_show_is_a_change() {
    let h = hooks();
    h.write("task_done\r.lua", "return function() end");
    assert!(h.changes_with(|h| h.write("task_done\r.lua", "os.exit(1)")));
}

#[test]
fn a_helper_replaced_by_a_link_is_a_change_and_the_link_is_never_followed() {
    let h = hooks();
    h.write("lib/real.lua", "return {}");
    assert!(h.changes_with(|h| {
        std::fs::remove_file(h.at("lib/helpers.lua")).unwrap();
        std::os::unix::fs::symlink("real.lua", h.at("lib/helpers.lua")).unwrap();
    }));
    let linked = h.fingerprint();
    h.write("lib/real.lua", "return { other = true }");
    let fingerprint = h.fingerprint().unwrap();
    let kept = fingerprint.files();
    assert_eq!(
        kept.get(".damhooks/lib/helpers.lua"),
        linked.unwrap().files().get(".damhooks/lib/helpers.lua")
    );
}

#[test]
fn a_hook_file_that_is_a_link_is_never_loaded_and_is_counted() {
    let h = hooks();
    let marker = h.0.marker();
    h.write(
        "lib/marking.lua",
        &format!("io.open({marker:?}, 'w'):close()\nreturn function() end\n"),
    );
    std::os::unix::fs::symlink("lib/marking.lua", h.at("task_done.lua")).unwrap();
    let read = ProjectHookFolder::read(&h.0.path("app")).unwrap();
    let loaded = h.0.file().project_hooks(&read, "Ada").unwrap();
    assert_eq!(loaded.hooks.len(), 1);
    assert!(!marker.exists());
    assert_eq!(read.names().1, 1);
}

#[test]
fn a_folder_too_large_to_check_has_no_fingerprint() {
    let h = hooks();
    for n in 0..1001 {
        h.write(&format!("lib/{n}.lua"), "");
    }
    assert_eq!(h.fingerprint(), None);
}
