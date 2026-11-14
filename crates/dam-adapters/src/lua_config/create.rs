use std::fs::DirBuilder;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::DirBuilderExt;

use super::file::LuaConfigFile;
use super::write::folder_of;

const TEMPLATE: &str = include_str!("template.lua");

impl LuaConfigFile {
    pub fn create_if_missing(&self) -> std::io::Result<bool> {
        self.write_the_template_once()
            .map_err(without_the_temporary_path)
    }

    fn write_the_template_once(&self) -> std::io::Result<bool> {
        let path = self.config_path();
        if path.try_exists()? {
            return Ok(false);
        }
        let dir = folder_of(path);
        DirBuilder::new().recursive(true).mode(0o700).create(&dir)?;
        let mut file = tempfile::NamedTempFile::new_in(&dir)?;
        file.write_all(TEMPLATE.as_bytes())?;
        file.as_file().sync_all()?;
        match file.persist_noclobber(path) {
            Ok(_) => Ok(true),
            Err(e) if e.error.kind() == ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(e.error),
        }
    }
}

fn without_the_temporary_path(error: std::io::Error) -> std::io::Error {
    match error.raw_os_error() {
        Some(code) => std::io::Error::from_raw_os_error(code),
        None => std::io::Error::from(error.kind()),
    }
}
