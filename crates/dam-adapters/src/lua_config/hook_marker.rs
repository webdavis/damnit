use dam_application::{HookEvent, STARTED_BY_HOOK_VARIABLE};
use mlua::{Lua, Table, Value};

const MARKER: &str = "hook_marker";

const MARK_WHAT_A_HOOK_STARTS: &str = r#"
local variable, marker = ...
local execute, popen = os.execute, io.popen
local function marked(command)
  if marker.event == nil then return command end
  return "export " .. variable .. "=" .. marker.event .. "; " .. command
end
os.execute = function(command, ...)
  if command == nil then return execute() end
  return execute(marked(command), ...)
end
io.popen = function(command, ...)
  return popen(marked(command), ...)
end
"#;

pub(super) fn mark_what_a_hook_starts(lua: &Lua) -> mlua::Result<()> {
    let marker = lua.create_table()?;
    lua.set_named_registry_value(MARKER, &marker)?;
    lua.load(MARK_WHAT_A_HOOK_STARTS)
        .call::<()>((STARTED_BY_HOOK_VARIABLE, marker))
}

pub(super) fn while_marked<T>(
    lua: &Lua,
    event: HookEvent,
    run: impl FnOnce() -> mlua::Result<T>,
) -> mlua::Result<T> {
    let marker: Table = lua.named_registry_value(MARKER)?;
    marker.set("event", event.as_str())?;
    let ran = run();
    marker.set("event", Value::Nil)?;
    ran
}
