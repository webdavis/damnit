use dam_application::RemoteName;

use super::support::Folder;

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
