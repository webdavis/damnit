use std::path::{Path, PathBuf};

pub trait ProjectFolders {
    fn holds_project_file(&self, folder: &Path) -> bool;
}

pub fn find_project(
    start: &Path,
    personal_config: &Path,
    folders: &dyn ProjectFolders,
) -> Option<PathBuf> {
    start
        .ancestors()
        .filter(|folder| *folder != personal_config)
        .find(|folder| folders.holds_project_file(folder))
        .map(Path::to_path_buf)
}
