use std::collections::HashMap;
use std::path::{Path, PathBuf};

use mlua::chunk::ChunkMode;
use mlua::{Function, Lua, Table, Value};

use super::hook_tree::{HookTree, TreeEntry};

pub(super) fn require_only_from(lua: &Lua, root: &Path, tree: &HookTree) -> mlua::Result<()> {
    let modules: HashMap<String, (PathBuf, Vec<u8>)> = tree
        .entries
        .iter()
        .filter_map(|(relative, entry)| match entry {
            TreeEntry::File(bytes) => Some((
                relative.to_str()?.to_string(),
                (root.join(relative), bytes.clone()),
            )),
            _ => None,
        })
        .collect();
    let package: Table = lua.globals().get("package")?;
    let root_text = root.display();
    package.set("path", format!("{root_text}/?.lua;{root_text}/?/init.lua"))?;
    package.set("cpath", "")?;
    let searcher = lua.create_function(move |lua, name: String| {
        let base = name.replace('.', "/");
        for relative in [format!("{base}.lua"), format!("{base}/init.lua")] {
            if let Some((path, bytes)) = modules.get(&relative) {
                let chunk = lua
                    .load(bytes.as_slice())
                    .set_name(format!("@{}", path.display()))
                    .set_mode(ChunkMode::Text)
                    .into_function()?;
                let found = lua.create_string(path.as_os_str().as_encoded_bytes())?;
                return Ok((Value::Function(chunk), Value::String(found)));
            }
        }
        let missing = format!("\n\tno module '{name}' among the files dam checked in .damhooks");
        Ok((Value::String(lua.create_string(missing)?), Value::Nil))
    })?;
    let preload: Function = package.get::<Table>("searchers")?.get(1)?;
    package.set("searchers", lua.create_sequence_from([preload, searcher])?)?;
    Ok(())
}
