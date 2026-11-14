use std::sync::Barrier;

use dam_application::{ConfigFile, EditedTable, RemoteName};

use super::support::{Folder, bare_token, held_already};

fn linked(folder: &Folder, name: &str) {
    std::fs::create_dir_all(folder.path("real")).unwrap();
    std::fs::write(folder.path(&format!("real/{name}")), "return {}\n").unwrap();
    std::os::unix::fs::symlink(format!("real/{name}"), folder.path(name)).unwrap();
}

fn is_link(folder: &Folder, name: &str) -> bool {
    let link = std::fs::symlink_metadata(folder.path(name)).unwrap();
    link.file_type().is_symlink()
}

#[test]
fn a_remote_added_through_a_symbolic_link_lands_in_the_file_it_points_to() {
    let folder = Folder::new();
    linked(&folder, "config.lua");
    let todoist = RemoteName::parse("todoist").unwrap();
    folder.file().add_remote(&todoist, "todoist::").unwrap();
    assert!(is_link(&folder, "config.lua"));
    assert!(folder.text("real/config.lua").contains("todoist::"));
}

#[test]
fn a_token_added_through_a_symbolic_link_lands_in_the_file_it_points_to() {
    let folder = Folder::new();
    linked(&folder, "secrets.lua");
    folder.file().add_token(&bare_token("tool")).unwrap();
    assert!(is_link(&folder, "secrets.lua"));
    assert!(folder.text("real/secrets.lua").contains("tool = {"));
}

#[test]
fn two_edits_racing_for_one_name_let_exactly_one_through() {
    for _ in 0..8 {
        let folder = Folder::new();
        let start = Barrier::new(2);
        let refusals: Vec<_> = std::thread::scope(|scope| {
            let racers: Vec<_> = (0..2)
                .map(|_| {
                    scope.spawn(|| {
                        start.wait();
                        folder.file().add_token(&bare_token("tool"))
                    })
                })
                .collect();
            racers
                .into_iter()
                .filter_map(|racer| racer.join().unwrap().err())
                .collect()
        });
        assert_eq!(refusals, [held_already(EditedTable::Tokens, "tool")]);
        assert_eq!(folder.load().unwrap().tokens.names, ["tool"]);
        assert_eq!(folder.text("secrets.lua").matches("tool = {").count(), 1);
    }
}

fn files_in(folder: &Folder, dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder.path(dir))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn edit_both(folder: &Folder) {
    let todoist = RemoteName::parse("todoist").unwrap();
    folder.file().add_remote(&todoist, "todoist::").unwrap();
    folder.file().add_token(&bare_token("tool")).unwrap();
    folder.file().remove_token("tool").unwrap();
}

#[test]
fn edits_leave_no_file_beside_the_two_they_edit() {
    let folder = Folder::new();
    edit_both(&folder);
    assert_eq!(files_in(&folder, ""), ["config.lua", "secrets.lua"]);
}

#[test]
fn edits_through_links_leave_no_file_beside_the_links_or_their_targets() {
    let folder = Folder::new();
    linked(&folder, "config.lua");
    linked(&folder, "secrets.lua");
    edit_both(&folder);
    assert_eq!(files_in(&folder, ""), ["config.lua", "real", "secrets.lua"]);
    assert_eq!(files_in(&folder, "real"), ["config.lua", "secrets.lua"]);
}
