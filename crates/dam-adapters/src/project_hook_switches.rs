use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use dam_application::{ConfigError, ProjectHookSwitches};

const RECORD: &str = "project-hooks.json";

pub struct ProjectHookSwitchFile {
    record: PathBuf,
}

impl ProjectHookSwitchFile {
    pub fn beside_the_store(store: &Path) -> ProjectHookSwitchFile {
        ProjectHookSwitchFile {
            record: store.with_file_name(RECORD),
        }
    }

    fn read(&self) -> Result<BTreeMap<String, bool>, ConfigError> {
        let unreadable = |why: String| ConfigError::Unreadable {
            path: self.record.clone(),
            why,
        };
        match std::fs::read_to_string(&self.record) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| unreadable(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(unreadable(e.to_string())),
        }
    }

    fn write(&self, switches: &BTreeMap<String, bool>) -> Result<(), ConfigError> {
        let unwritable = |e: std::io::Error| ConfigError::Unwritable {
            path: self.record.clone(),
            why: e.to_string(),
        };
        let folder = self.record.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(folder).map_err(unwritable)?;
        let mut file = tempfile::NamedTempFile::new_in(folder).map_err(unwritable)?;
        let text = serde_json::to_string_pretty(switches).map_err(std::io::Error::other);
        file.write_all(text.map_err(unwritable)?.as_bytes())
            .map_err(unwritable)?;
        file.persist(&self.record)
            .map_err(|e| unwritable(e.error))?;
        Ok(())
    }
}

fn key_of(project: &Path) -> String {
    std::fs::canonicalize(project)
        .unwrap_or_else(|_| project.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

impl ProjectHookSwitches for ProjectHookSwitchFile {
    fn switch(&self, project: &Path) -> Result<Option<bool>, ConfigError> {
        Ok(self.read()?.get(&key_of(project)).copied())
    }

    fn turn(&self, project: &Path, on: Option<bool>) -> Result<(), ConfigError> {
        let mut switches = self.read()?;
        match on {
            Some(on) => switches.insert(key_of(project), on),
            None => switches.remove(&key_of(project)),
        };
        self.write(&switches)
    }
}

#[cfg(test)]
mod tests;
