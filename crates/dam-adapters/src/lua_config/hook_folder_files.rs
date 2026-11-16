use std::path::{Path, PathBuf};

use dam_application::ConfigError;

use super::evaluate::ErrorText;
use super::hook_folder::lua_files;

pub(in crate::lua_config) struct HookFolder<'a> {
    pub(super) path: &'a Path,
    pub(super) files: Vec<PathBuf>,
    pub(super) unshown: usize,
    pub(super) error_text: ErrorText,
}

impl HookFolder<'_> {
    pub(in crate::lua_config) fn personal(path: &Path) -> Result<HookFolder<'_>, ConfigError> {
        Ok(HookFolder {
            path,
            files: lua_files(path)?,
            unshown: 0,
            error_text: ErrorText::QuotesTheFile,
        })
    }

    pub(in crate::lua_config) fn project(path: &Path) -> Result<HookFolder<'_>, ConfigError> {
        let (files, unshown): (Vec<PathBuf>, Vec<PathBuf>) =
            lua_files(path)?.into_iter().partition(|file| shown(file));
        Ok(HookFolder {
            path,
            files,
            unshown: unshown.len(),
            error_text: ErrorText::NeverQuotesTheFile,
        })
    }
}

fn shown(file: &Path) -> bool {
    file.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| !name.chars().any(char::is_control))
}
