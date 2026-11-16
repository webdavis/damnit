use std::path::Path;

use dam_application::{ConfigError, HookConfig, HookEvent, HookFile, Hooks, PulledHook};

use super::evaluate::stopped_at_an_error;
use super::file::LuaConfigFile;
use super::functions::LuaFunctions;
use super::hook_folder::{
    HookFolder, PROJECT_HOOKS_FOLDER, configured, list_hook_folder, name_hook_folder,
    read_hook_folder,
};

pub type ProjectHookNames = (Vec<(String, Option<HookEvent>)>, usize);

pub struct ProjectHookFiles {
    pub hooks: Vec<HookConfig>,
    pub task_pulled: Option<PulledHook>,
    pub run: Box<dyn Hooks>,
}

impl LuaConfigFile {
    pub fn project_hooks(
        &self,
        project: &Path,
        owner: &str,
    ) -> Result<ProjectHookFiles, ConfigError> {
        let lua = self.fenced_lua()?;
        let folder = project.join(PROJECT_HOOKS_FOLDER);
        let kept = read_hook_folder(&lua, &HookFolder::project(&folder)?)?;
        let (hooks, task_pulled) = configured(&kept);
        let run = LuaFunctions::keep(lua, folder.clone(), None, kept, owner, self.function_limit)
            .map_err(|_| stopped_at_an_error(&folder))?;
        Ok(ProjectHookFiles {
            hooks,
            task_pulled,
            run: Box::new(run),
        })
    }

    pub fn project_hook_files(
        &self,
        project: &Path,
    ) -> Result<(Vec<HookFile>, usize), ConfigError> {
        let path = project.join(PROJECT_HOOKS_FOLDER);
        let folder = HookFolder::project(&path)?;
        Ok((
            list_hook_folder(&self.fenced_lua()?, &folder),
            folder.unshown,
        ))
    }

    pub fn project_hook_names(project: &Path) -> Result<ProjectHookNames, ConfigError> {
        let path = project.join(PROJECT_HOOKS_FOLDER);
        let folder = HookFolder::project(&path)?;
        Ok((name_hook_folder(&folder), folder.unshown))
    }
}
