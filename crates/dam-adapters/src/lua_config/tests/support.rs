use dam_application::{Config, ConfigError};
use tempfile::TempDir;

use crate::lua_config::LuaConfigFile;

pub(super) struct Folder {
    dir: TempDir,
}

impl Folder {
    pub(super) fn new() -> Folder {
        Folder {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub(super) fn with_config(self, text: &str) -> Folder {
        std::fs::write(self.dir.path().join("config.lua"), text).unwrap();
        self
    }

    pub(super) fn file(&self) -> LuaConfigFile {
        LuaConfigFile::new(self.dir.path().join("config.lua"))
    }

    pub(super) fn load(&self) -> Result<Config, ConfigError> {
        self.file().load()
    }

    pub(super) fn refusal(&self) -> String {
        self.load().unwrap_err().to_string()
    }
}

pub(super) fn config(text: &str) -> Folder {
    Folder::new().with_config(text)
}
