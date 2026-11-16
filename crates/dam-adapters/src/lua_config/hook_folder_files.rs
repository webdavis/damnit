use std::path::{Path, PathBuf};

use dam_application::ConfigError;

use super::evaluate::ErrorText;
use super::hook_folder::lua_files;
use super::hook_tree::{HookTree, TreeEntry};

pub(in crate::lua_config) struct HookFolder {
    pub(super) path: PathBuf,
    pub(super) files: Vec<HookSource>,
    pub(super) unshown: usize,
    pub(super) error_text: ErrorText,
}

pub(in crate::lua_config) struct HookSource {
    pub(super) path: PathBuf,
    pub(super) bytes: std::io::Result<Vec<u8>>,
}

impl HookFolder {
    pub(in crate::lua_config) fn personal(path: &Path) -> Result<HookFolder, ConfigError> {
        Ok(HookFolder {
            path: path.to_path_buf(),
            files: read(lua_files(path)?),
            unshown: 0,
            error_text: ErrorText::QuotesTheFile,
        })
    }

    pub(in crate::lua_config) fn project(path: &Path, tree: &HookTree) -> HookFolder {
        let mut folder = HookFolder {
            path: path.to_path_buf(),
            files: Vec::new(),
            unshown: 0,
            error_text: ErrorText::NeverQuotesTheFile,
        };
        let hook_files = tree.entries.iter().filter(|(relative, _)| {
            relative.parent() == Some(Path::new("")) && relative.extension() == Some("lua".as_ref())
        });
        for (relative, entry) in hook_files {
            match entry {
                TreeEntry::File(bytes) if shown(relative) => folder.files.push(HookSource {
                    path: path.join(relative),
                    bytes: Ok(bytes.clone()),
                }),
                TreeEntry::Folder => {}
                _ => folder.unshown += 1,
            }
        }
        folder
    }
}

impl HookSource {
    pub(super) fn text(&self) -> std::io::Result<&str> {
        let bytes = self.bytes.as_ref().map_err(clone_of)?;
        std::str::from_utf8(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

fn clone_of(error: &std::io::Error) -> std::io::Error {
    std::io::Error::new(error.kind(), error.to_string())
}

fn read(files: Vec<PathBuf>) -> Vec<HookSource> {
    files
        .into_iter()
        .map(|path| HookSource {
            bytes: std::fs::read(&path),
            path,
        })
        .collect()
}

fn shown(file: &Path) -> bool {
    file.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| !name.chars().any(char::is_control))
}
