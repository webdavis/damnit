use std::path::{Path as FsPath, PathBuf};

use dam_adapters::{FileSystemProjectFolders, read_project_file};
use dam_application::{ConfigError, PROJECT_FILE, Project, find_project};
use dam_domain::Path;

use crate::args::ScopeArgs;
use crate::error::{CliError, config_text};

pub(crate) struct RunFolders {
    pub(crate) folder: PathBuf,
    pub(crate) personal_config: PathBuf,
}

impl RunFolders {
    pub(crate) fn of_this_run(config_path: &FsPath) -> Result<RunFolders, CliError> {
        Ok(RunFolders::new(std::env::current_dir()?, config_path))
    }

    pub(crate) fn new(folder: PathBuf, config_path: &FsPath) -> RunFolders {
        let beside_config = config_path.parent().unwrap_or(config_path);
        RunFolders {
            folder,
            personal_config: std::fs::canonicalize(beside_config)
                .unwrap_or_else(|_| beside_config.into()),
        }
    }

    pub(crate) fn standing(&self) -> Standing {
        Standing::of(&self.folder, &self.personal_config)
    }
}

#[derive(Debug)]
pub(crate) enum Standing {
    Outside,
    Inside(Project),
    Unreadable(ConfigError),
}

impl Standing {
    pub(crate) fn of(folder: &FsPath, personal_config: &FsPath) -> Standing {
        let Some(folder) = find_project(folder, personal_config, &FileSystemProjectFolders) else {
            return Standing::Outside;
        };
        match read_project_file(&folder.join(PROJECT_FILE)) {
            Ok(settings) => Standing::Inside(Project { folder, settings }),
            Err(e) => Standing::Unreadable(e),
        }
    }

    pub(crate) fn warning(&self) -> Option<String> {
        let Standing::Unreadable(error) = self else {
            return None;
        };
        let text = config_text(error).replace('\n', " ");
        Some(format!("dam: warning: {text}; run dam doctor config"))
    }

    pub(crate) fn place(self, scope: &ScopeArgs) -> Result<Path, CliError> {
        if let Some(project) = &scope.project {
            return Ok(project.clone());
        }
        if scope.global {
            return Ok(Path::root());
        }
        match self {
            Standing::Outside => Ok(Path::root()),
            Standing::Inside(project) => Path::parse(&project.settings.path).map_err(|_| {
                CliError::Usage(format!(
                    "path in {} is not a path such as work/app/; correct it",
                    project.file().display()
                ))
            }),
            Standing::Unreadable(e) => Err(CliError::Config(e)),
        }
    }
}

#[cfg(test)]
mod tests;
