use std::path::Path;

use dam_application::{ConfigError, HookConfig, Hooks, PulledHook};

use super::evaluate::stopped_at_an_error;
use super::file::LuaConfigFile;
use super::functions::LuaFunctions;
use super::hook_folder::{PROJECT_HOOKS_FOLDER, configured, read_hook_folder};

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
        let kept = read_hook_folder(&lua, &folder)?;
        let (hooks, task_pulled) = configured(&kept);
        let run = LuaFunctions::keep(lua, folder.clone(), None, kept, owner, self.function_limit)
            .map_err(|_| stopped_at_an_error(&folder))?;
        Ok(ProjectHookFiles {
            hooks,
            task_pulled,
            run: Box::new(run),
        })
    }
}
