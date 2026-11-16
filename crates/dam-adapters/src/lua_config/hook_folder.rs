use std::path::{Path, PathBuf};

use dam_application::{ConfigError, HookConfig, HookEvent, PulledHook};
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
    let mut hooks = paths
        .into_iter()
        .map(|path| read_hook_file(lua, path))
        .collect::<Result<Vec<_>, _>>()?;
    hooks.sort_by_key(|hook| hook.event);
    Ok(hooks)
}

fn read_hook_file(lua: &Lua, path: PathBuf) -> Result<KeptHook, ConfigError> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let file = format!("{HOOKS_FOLDER}/{name}");
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let event =
        event_named(&stem).ok_or_else(|| ConfigError::UnknownHookFile { file: file.clone() })?;
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
