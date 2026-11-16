use std::path::{Path, PathBuf};

use dam_application::{ConfigError, HookConfig, HookEvent, HookFile, HookSetting, PulledHook};
use mlua::Lua;

use super::evaluate::{ErrorText, evaluated};
use super::hook::{ReadHook, event_named, read_hook};

pub(super) const HOOKS_FOLDER: &str = "hooks";

pub(super) struct KeptHook {
    pub(super) event: HookEvent,
    pub(super) path: PathBuf,
    pub(super) read: ReadHook,
    file: String,
}

pub(super) fn read_hook_folder(lua: &Lua, folder: &Path) -> Result<Vec<KeptHook>, ConfigError> {
    let mut hooks = lua_files(folder)?
        .into_iter()
        .map(|path| read_hook_file(lua, path))
        .collect::<Result<Vec<_>, _>>()?;
    hooks.sort_by_key(|hook| hook.event);
    Ok(hooks)
}

pub(super) fn list_hook_folder(lua: &Lua, folder: &Path) -> Result<Vec<HookFile>, ConfigError> {
    let mut files: Vec<HookFile> = lua_files(folder)?
        .into_iter()
        .map(|path| HookFile {
            file: file_of(&path),
            event: event_of(&path),
            read: read_hook_file(lua, path).map(|kept| kept.setting()),
        })
        .collect();
    files.sort_by_key(|hook| (hook.event.is_none(), hook.event, hook.file.clone()));
    Ok(files)
}

fn lua_files(folder: &Path) -> Result<Vec<PathBuf>, ConfigError> {
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

fn file_of(path: &Path) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    format!("{HOOKS_FOLDER}/{name}")
}

fn event_of(path: &Path) -> Option<HookEvent> {
    event_named(&path.file_stem().unwrap_or_default().to_string_lossy())
}

fn read_hook_file(lua: &Lua, path: PathBuf) -> Result<KeptHook, ConfigError> {
    let file = file_of(&path);
    let event =
        event_of(&path).ok_or_else(|| ConfigError::UnknownHookFile { file: file.clone() })?;
    let text = std::fs::read_to_string(&path).map_err(|e| unreadable(&path, &e))?;
    let value = evaluated(lua, &path, &text, ErrorText::QuotesTheFile)?;
    let read = read_hook(&file, event, value)?;
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
