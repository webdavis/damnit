-- dam config. dam made this file because none was here. It stays yours: `dam remote add` adds a
-- remote to it and keeps the rest. Every setting below is commented out or set to its default.
-- Remove the leading -- from an example line to use it.
--
-- Secrets never go here, so this file is always safe to show. They live in secrets.lua beside it,
-- a Lua file that returns a table too:
--   return {
--     schema = 1,
--     remote = {
--       todoist = { api_token = function() return os.getenv("TODOIST_API_TOKEN") end },
--     },
--     token = { ... },
--   }
-- A remote's credential there is a string, or a function dam calls only when the remote is used.
--
-- Its token table holds what `dam token create` writes; `dam token list` shows the tokens and
-- `dam token revoke` removes one. dam edits only that table and keeps the rest as written. A
-- token lets a tool run dam as itself: it sets DAM_TOKEN to the value. A token takes value,
-- not_before and expires (RFC 3339 times), via (local, mcp or both; local by default), require
-- (the details each request must state with --meta KEY=VALUE, such as session and model), and
-- grant, a list of grants.
-- Each grant allows the permissions in allow, and only within all of its limits that are set:
--   paths, objects under one of these paths.
--   labels, objects carrying one of these labels.
--   own, objects a token of this name made, and its own waiting items. A create always passes.
--   remotes, the remotes named here.
-- One grant must cover everything a command touches.
-- The permissions are task:read, task:create, task:change, task:complete, task:move,
-- task:remove, event:read, event:create, event:change, event:move, event:remove,
-- pending:read, pending:accept, pending:reject, working:read, stage:read, history:read,
-- history:commit, remote:read, remote:pull, remote:push, conflict:read, conflict:resolve
-- and config:read.
return {
  -- The name your own tasks, events and changes carry as their author. Default: owner.
  --owner = "Ada",

  -- With interactive = true, `dam done` asks what to do with a task's open children and
  -- dependencies instead of refusing.
  --done = { interactive = true },

  -- With progress = false, a slow step shows its progress line only under --progress.
  --progress = false,

  -- With confirm_above = 10, a - or --filter run of edit, done, mv or rm over more than 10
  -- objects asks first, and needs --expect without a terminal or with ids piped in. Default: never ask.
  --confirm_above = 10,

  -- A remote dam pulls from and pushes to, run by the dam-remote-<helper> program on PATH.
  -- url is <helper>::<address>, a helper name being letters, digits, - and _. deadline is the
  -- longest the helper may stay silent, 60s by default, a whole number then s, m or h. path limits
  -- a push to the objects under that path. except lists paths a push never sends, nor the task that
  -- holds each, and wins over path. sync.interval is how often background sync runs, 15m by default
  -- and 1m at least. sync.allowed = false means background sync may never start here.
  -- Nothing syncs by itself; dam sync run syncs a remote now.
  --remote = {
    --todoist = {
      --url = "todoist::",
      --deadline = "60s",
      --sync = { interval = "15m", allowed = true },
      --path = "work",
      --except = { "work/private" },
    --},
  --},

  -- A label category groups labels so a query can say effort:deep. With exclusive = true an
  -- object carries at most one of its values.
  --category = {
    --effort = { values = { "light", "admin", "deep" }, exclusive = true },
  --},

  -- A saved filter runs by name with dam list --filter NAME. Each key is a dam list flag and
  -- its value the text you would type after it; a list repeats the flag, and any holds tables
  -- of which at least one must hold. A match function that returns true to keep an object
  -- narrows what the fields list. today lists tasks not done that are due today or before.
  --filter = {
    --today = { due = "..today" },
    --undated = { kind = "task", due = "none", match = function(task, dam) return task.task.deadline == nil end },
  --},
}
