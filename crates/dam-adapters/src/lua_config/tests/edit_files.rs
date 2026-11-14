use std::sync::Barrier;

use dam_application::RemoteName;

use super::support::{Folder, bare_token};

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
        let refusals: Vec<String> = std::thread::scope(|scope| {
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
                .map(|why| why.to_string())
                .collect()
        });
        assert_eq!(refusals, ["config: secrets.lua already holds token tool"]);
        assert_eq!(folder.load().unwrap().tokens.names, ["tool"]);
        assert_eq!(folder.text("secrets.lua").matches("tool = {").count(), 1);
    }
}
