use mlua::{Lua, Table};

use super::evaluate::ErrorText;

const KEEP_THE_REAL_ONES: &str = "return { print = print, write = io.write, \
     output = io.output, stdout = io.stdout, stderr = io.stderr }";

const DISCARD: &str = r#"
local sink = {}
setmetatable(sink, { __index = function() return function() return sink end end })
print = function() end
io.write = function() return sink end
io.output = function() return sink end
io.stdout = sink
io.stderr = sink
"#;

const TO_STANDARD_ERROR: &str = r##"
local stderr = io.stderr
print = function(...)
  local parts = {}
  for i = 1, select("#", ...) do parts[i] = tostring((select(i, ...))) end
  stderr:write(table.concat(parts, "\t"), "\n")
end
io.write = function(...) return stderr:write(...) end
io.output = function() return stderr end
io.stdout = stderr
"##;

const PUT_BACK: &str = r#"
local real = ...
print, io.write, io.output = real.print, real.write, real.output
io.stdout, io.stderr = real.stdout, real.stderr
"#;

pub(super) fn while_the_top_level_runs<T>(
    lua: &Lua,
    error_text: ErrorText,
    run: impl FnOnce() -> mlua::Result<T>,
) -> mlua::Result<T> {
    let replacement = match error_text {
        ErrorText::QuotesTheFile => return run(),
        ErrorText::NeverQuotesTheFile => DISCARD,
        ErrorText::QuotesWhatItsCodeRaised => TO_STANDARD_ERROR,
    };
    let real: Table = lua.load(KEEP_THE_REAL_ONES).eval()?;
    lua.load(replacement).exec()?;
    let ran = run();
    let finalizers_ran = lua.gc_collect().and_then(|()| lua.gc_collect());
    lua.load(PUT_BACK).call::<()>(real)?;
    finalizers_ran.and(ran)
}
