use std::sync::atomic::{AtomicU32, Ordering};

use mlua::debug::Debug;
use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Value, VmState};

const KEPT_GLOBALS: [&str; 15] = [
    "_G", "_VERSION", "assert", "error", "ipairs", "next", "pairs", "rawequal", "rawget", "rawlen",
    "rawset", "select", "tonumber", "tostring", "type",
];
const MEMORY_LIMIT: usize = 8 * 1024 * 1024;
const INSTRUCTIONS_PER_CHECK: u32 = 1_000;
const CHECKS_ALLOWED: u32 = 10_000;

pub(super) fn locked_lua() -> mlua::Result<Lua> {
    let lua = Lua::new_with(StdLib::NONE, LuaOptions::default())?;
    lua.set_memory_limit(MEMORY_LIMIT)?;
    let globals = lua.globals();
    let names = globals
        .pairs::<Value, Value>()
        .map(|pair| pair.map(|(name, _)| name))
        .collect::<mlua::Result<Vec<_>>>()?;
    for name in names {
        let kept = name
            .as_string()
            .is_some_and(|name| KEPT_GLOBALS.iter().any(|kept| name == *kept));
        if !kept {
            globals.raw_remove(name)?;
        }
    }
    let checks = AtomicU32::new(0);
    lua.set_global_hook(every(INSTRUCTIONS_PER_CHECK), move |lua, debug| {
        match checks.fetch_add(1, Ordering::Relaxed) < CHECKS_ALLOWED {
            true => Ok(VmState::Continue),
            false => ran_over(lua, debug),
        }
    })?;
    Ok(lua)
}

fn every(instructions: u32) -> HookTriggers {
    HookTriggers::new().every_nth_instruction(instructions)
}

fn ran_over(lua: &Lua, _: &Debug) -> mlua::Result<VmState> {
    lua.set_global_hook(every(1), ran_over)?;
    Err(mlua::Error::runtime("ran over its instruction limit"))
}
