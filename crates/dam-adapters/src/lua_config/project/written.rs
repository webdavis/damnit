use std::io::Write;
use std::path::{Path, PathBuf};

use dam_application::{ConfigError, PROJECT_FILE, ProjectFolders, ProjectSettings};

use super::super::write::string_literal;

pub struct FileSystemProjectFolders;

impl ProjectFolders for FileSystemProjectFolders {
    fn holds_project_file(&self, folder: &Path) -> bool {
        folder.join(PROJECT_FILE).is_file()
    }
}

pub fn write_project_file(
    folder: &Path,
    settings: &ProjectSettings,
) -> Result<PathBuf, ConfigError> {
    let file = folder.join(PROJECT_FILE);
    let unwritable = |error: std::io::Error| ConfigError::Unwritable {
        path: file.clone(),
        why: error.to_string(),
    };
    let text = format!("return {{ path = {} }}\n", string_literal(&settings.path));
    std::fs::File::create_new(&file)
        .and_then(|mut opened| opened.write_all(text.as_bytes()))
        .map_err(unwritable)?;
    Ok(file)
}
