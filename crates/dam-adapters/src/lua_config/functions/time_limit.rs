use std::time::Instant;

use mlua::debug::Debug;
use mlua::{HookTriggers, Lua, VmState};

const INSTRUCTIONS_BETWEEN_CLOCK_READS: u32 = 10_000;

struct Deadline(Instant);

pub(in crate::lua_config) fn limit_every_thread(lua: &Lua) -> mlua::Result<()> {
    checking_every(lua, INSTRUCTIONS_BETWEEN_CLOCK_READS)
}

pub(super) fn limit_to(lua: &Lua, deadline: Instant) -> mlua::Result<()> {
    lua.set_app_data(Deadline(deadline));
    limit_every_thread(lua)
}

pub(super) fn lift(lua: &Lua) {
    lua.remove_app_data::<Deadline>();
}

fn checking_every(lua: &Lua, instructions: u32) -> mlua::Result<()> {
    let triggers = HookTriggers::new().every_nth_instruction(instructions);
    lua.set_global_hook(triggers, ran_over)
}

fn ran_over(lua: &Lua, _: &Debug) -> mlua::Result<VmState> {
    let passed = lua
        .app_data_ref::<Deadline>()
        .is_some_and(|deadline| Instant::now() >= deadline.0);
    if !passed {
        return Ok(VmState::Continue);
    }
    // Raising on every instruction from here on leaves a pcall nothing to resume.
    checking_every(lua, 1)?;
    Err(mlua::Error::runtime("ran over its time limit"))
}
