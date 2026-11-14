# The config.lua state lives as long as the loaded config

## Context

A filter's `match` is a Lua function written in `config.lua`. dam calls it after the file has run,
once for each object a listing would show. A function can be called only in the Lua state that
made it, so the state that ran `config.lua` cannot be dropped once the settings are read.

## Decision

The Lua state that ran `config.lua` is kept for the life of the loaded config.
`LuaFilterFunctions` holds it and calls each `match` in it. Every listing in that process uses the
same state.

## Consequence

A `match` can set globals and change the settings table. What one call leaves there, the next call
in the same state sees. A process that runs one command for one caller, as the `dam` command does,
is not affected.

A process that serves more than one caller must give each caller a state that another caller's
filter cannot have changed. It can run `config.lua` again for each caller, or keep one state per
caller. It must not share one state between callers.

Where the code says it: `crates/dam-adapters/src/lua_config/filter_function.rs` and
`crates/dam-adapters/src/lua_config/file.rs`.
