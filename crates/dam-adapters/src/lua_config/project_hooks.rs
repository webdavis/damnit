use std::path::Path;

use dam_application::{
    ConfigError, HookConfig, HookEvent, HookFile, HookFingerprint, Hooks, PulledHook,
};

use super::evaluate::stopped_at_an_error;
use super::file::LuaConfigFile;
use super::functions::LuaFunctions;
use super::hook_folder::{
    HookFolder, PROJECT_HOOKS_FOLDER, configured, list_hook_folder, name_hook_folder,
    read_hook_folder,
};
use super::hook_tree::{HookTree, read_tree};
use super::project_require::require_only_from;
use mlua::Lua;

pub struct ProjectHookFolder {
    hooks: HookFolder,
    tree: HookTree,
}

pub struct ProjectHookFiles {
    pub hooks: Vec<HookConfig>,
    pub task_pulled: Option<PulledHook>,
    pub run: Box<dyn Hooks>,
}

impl ProjectHookFolder {
    pub fn read(project: &Path) -> Result<ProjectHookFolder, ConfigError> {
        let path = project.join(PROJECT_HOOKS_FOLDER);
        let tree = read_tree(&path)?;
        Ok(ProjectHookFolder {
            hooks: HookFolder::project(&path, &tree),
            tree,
        })
    }

    pub fn fingerprint(&self) -> Result<Option<HookFingerprint>, ConfigError> {
        Ok(self.tree.fingerprint(PROJECT_HOOKS_FOLDER))
    }

    pub fn names(&self) -> (Vec<(String, Option<HookEvent>)>, usize) {
        (name_hook_folder(&self.hooks), self.hooks.unshown)
    }

    fn fenced(&self, file: &LuaConfigFile) -> Result<Lua, ConfigError> {
        let lua = file.fenced_lua()?;
        require_only_from(&lua, &self.hooks.path, &self.tree)
            .map_err(|_| stopped_at_an_error(&self.hooks.path))?;
        Ok(lua)
    }
}

impl LuaConfigFile {
    pub fn project_hooks(
        &self,
        folder: &ProjectHookFolder,
        owner: &str,
    ) -> Result<ProjectHookFiles, ConfigError> {
        let lua = folder.fenced(self)?;
        let kept = read_hook_folder(&lua, &folder.hooks)?;
        let (hooks, task_pulled) = configured(&kept);
        let path = folder.hooks.path.clone();
        let run = LuaFunctions::keep(lua, path.clone(), None, kept, owner, self.function_limit)
            .map_err(|_| stopped_at_an_error(&path))?;
        Ok(ProjectHookFiles {
            hooks,
            task_pulled,
            run: Box::new(run),
        })
    }

    pub fn project_hook_files(
        &self,
        folder: &ProjectHookFolder,
    ) -> Result<(Vec<HookFile>, usize), ConfigError> {
        Ok((
            list_hook_folder(&folder.fenced(self)?, &folder.hooks),
            folder.hooks.unshown,
        ))
    }
}
