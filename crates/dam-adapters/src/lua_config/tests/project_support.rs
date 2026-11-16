use std::path::PathBuf;

use dam_application::{ConfigError, ProjectSettings};
use tempfile::TempDir;

use crate::lua_config::read_project_file;

pub(super) struct ProjectFolder(TempDir);

impl ProjectFolder {
    pub(super) fn with(text: &str) -> ProjectFolder {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".dam.lua"), text).unwrap();
        ProjectFolder(dir)
    }

    pub(super) fn file(&self) -> PathBuf {
        self.0.path().join(".dam.lua")
    }

    pub(super) fn holds(&self, name: &str) -> bool {
        self.0.path().join(name).exists()
    }

    pub(super) fn read(&self) -> Result<ProjectSettings, ConfigError> {
        self.read_as("0.2.0")
    }

    pub(super) fn read_as(&self, this_dam: &str) -> Result<ProjectSettings, ConfigError> {
        read_project_file(&self.file(), this_dam)
    }
}
