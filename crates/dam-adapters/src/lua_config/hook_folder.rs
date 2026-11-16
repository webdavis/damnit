use std::path::{Path, PathBuf};

use dam_application::{ConfigError, HookConfig, HookEvent, HookFile, HookSetting, PulledHook};
use mlua::Lua;

use super::evaluate::{ErrorText, evaluated};
use super::hook::{ReadHook, event_named, read_hook};
pub(super) use super::hook_folder_files::HookFolder;

pub(super) const HOOKS_FOLDER: &str = "hooks";
pub const PROJECT_HOOKS_FOLDER: &str = ".damhooks";

pub(super) struct KeptHook {
    pub(super) event: HookEvent,
    pub(super) path: PathBuf,
    pub(super) read: ReadHook,
    file: String,
}

pub(super) fn read_hook_folder(
    lua: &Lua,
    folder: &HookFolder,
) -> Result<Vec<KeptHook>, ConfigError> {
    let mut hooks = folder
        .files
        .iter()
        .map(|path| read_hook_file(lua, folder, path.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    hooks.sort_by_key(|hook| hook.event);
    Ok(hooks)
}

pub(super) fn list_hook_folder(lua: &Lua, folder: &HookFolder) -> Vec<HookFile> {
    let mut files: Vec<HookFile> = folder
        .files
        .iter()
        .map(|path| HookFile {
            file: file_of(folder.path, path),
            event: event_of(path),
            read: read_hook_file(lua, folder, path.clone()).map(|kept| kept.setting()),
        })
        .collect();
    files.sort_by_key(|hook| (hook.event.is_none(), hook.event, hook.file.clone()));
    files
}

pub(super) fn name_hook_folder(folder: &HookFolder) -> Vec<(String, Option<HookEvent>)> {
    let mut names: Vec<_> = folder
        .files
        .iter()
        .map(|path| (file_of(folder.path, path), event_of(path)))
        .collect();
    names.sort_by_key(|(file, event)| (event.is_none(), *event, file.clone()));
    names
}

pub(super) fn lua_files(folder: &Path) -> Result<Vec<PathBuf>, ConfigError> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(unreadable(folder, &e)),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| unreadable(folder, &e))?.path();
        if path.extension().is_some_and(|e| e == "lua") {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn file_of(folder: &Path, path: &Path) -> String {
    let folder = folder.file_name().unwrap_or_default().to_string_lossy();
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    format!("{folder}/{name}")
}

fn event_of(path: &Path) -> Option<HookEvent> {
    event_named(&path.file_stem().unwrap_or_default().to_string_lossy())
}

fn read_hook_file(lua: &Lua, folder: &HookFolder, path: PathBuf) -> Result<KeptHook, ConfigError> {
    let file = file_of(folder.path, &path);
    let event =
        event_of(&path).ok_or_else(|| ConfigError::UnknownHookFile { file: file.clone() })?;
    let text = std::fs::read_to_string(&path).map_err(|e| unreadable(&path, &e))?;
    let read = evaluated(lua, &path, &text, ErrorText::QuotesTheFile)
        .and_then(|value| read_hook(&file, event, value))
        .map_err(|error| folder.error_text.about_what_it_holds(&path, error))?;
    Ok(KeptHook {
        event,
        path,
        read,
        file,
    })
}

impl KeptHook {
    fn setting(&self) -> HookSetting {
        match self.event {
            HookEvent::TaskPulled => HookSetting::Changes(self.read.changes),
            _ => HookSetting::OnFailure(self.read.on_failure),
        }
    }
}

pub(super) fn configured(hooks: &[KeptHook]) -> (Vec<HookConfig>, Option<PulledHook>) {
    let local = hooks
        .iter()
        .filter(|hook| hook.event != HookEvent::TaskPulled)
        .map(|hook| HookConfig {
            event: hook.event,
            on_failure: hook.read.on_failure,
            file: hook.file.clone(),
        })
        .collect();
    let pulled = hooks
        .iter()
        .find(|hook| hook.event == HookEvent::TaskPulled)
        .map(|hook| PulledHook {
            changes: hook.read.changes,
            file: hook.file.clone(),
        });
    (local, pulled)
}

fn unreadable(path: &Path, error: &std::io::Error) -> ConfigError {
    ConfigError::Unreadable {
        path: path.to_path_buf(),
        why: error.to_string(),
    }
}
