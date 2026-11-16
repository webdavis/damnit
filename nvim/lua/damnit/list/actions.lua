local argv = require("damnit.kernel.argv")

local M = {}

---@class damnit.Action
---@field command string
---@field desc string
---@field group string
---@field flags? string[]
---@field prefill? string
---@field field? string
---@field one? boolean
---@field always_confirm? boolean

local ROWS = "task rows"

---@type table<string, damnit.Action>
local ACTIONS = {
  x = { command = "done", desc = "done" },
  X = { command = "edit", flags = { "--no-done" }, desc = "reopen" },
  gx = { command = "done", prefill = "", desc = "done with flags (--force, --children ...)" },
  s = { command = "add", desc = "stage" },
  dd = { command = "rm", always_confirm = true, desc = "remove, after the confirm float" },
  U = { command = "restore", always_confirm = true, desc = "set back to the last commit, after the confirm float" },
  cd = { command = "edit", prefill = "--due ", field = "due", desc = "change the due date" },
  cD = { command = "edit", prefill = "--deadline ", field = "deadline", desc = "change the deadline" },
  cl = { command = "edit", prefill = "--labels ", field = "labels", desc = "add labels" },
  cL = { command = "edit", prefill = "--unlabels ", field = "labels", desc = "take labels off" },
  cg = { command = "edit", prefill = "--category ", field = "category", desc = "change a category" },
  ce = { command = "edit", prefill = "--estimate ", field = "estimate", desc = "change the estimate" },
  cr = { command = "edit", prefill = "--recurrence ", field = "recurrence", desc = "change the recurrence" },
  cs = {
    command = "edit",
    prefill = "--subject ",
    field = "subject",
    one = true,
    desc = "change the subject (one row)",
  },
  cm = { command = "mv", prefill = "--path ", field = "path", desc = "move to a path" },
  cP = { command = "mv", prefill = "--parent ", field = "path", desc = "move under a parent" },
  cc = { command = "edit", prefill = "", desc = "change with any dam edit flags" },
}

for priority = 1, 4 do
  ACTIONS["p" .. priority] = {
    command = "edit",
    flags = { "--priority", tostring(priority) },
    desc = "priority " .. priority,
  }
end

local ORDER = { "x", "X", "gx", "p1", "p2", "p3", "p4", "cd", "cD", "cl", "cL", "cg", "ce", "cr", "cs", "cm", "cP" }
vim.list_extend(ORDER, { "cc", "dd", "U", "s" })

---@return string[]
function M.keys()
  return vim.list_slice(ORDER)
end

---@param key string
---@return { desc: string, group: string }
function M.spec(key)
  return { desc = ACTIONS[key].desc, group = ROWS }
end

---@param key string
---@param oids string[]
---@param answer? string the prompt's text
---@return damnit.Batch
function M.batch(key, oids, answer)
  local action = ACTIONS[key]
  local flags = answer and argv.words(answer) or vim.list_slice(action.flags or {})
  return { command = action.command, oids = oids, flags = flags }
end

---@param key string
---@param oids string[]
---@return boolean
function M.confirms(key, oids)
  local action = ACTIONS[key]
  return action.prefill == nil and (action.always_confirm == true or #oids > 1)
end

local function quoted(text)
  return '"' .. text:gsub('[\\"]', "\\%0") .. '"'
end

---@param key string
---@param object? damnit.Object the row under the cursor
---@return { title: string, prefill: string, field?: string, one?: boolean }|nil
function M.prompt(key, object)
  local action = ACTIONS[key]
  if not action.prefill then
    return nil
  end
  local prefill = action.prefill
  if action.one and object then
    prefill = prefill .. quoted(object.subject)
  end
  return { title = "dam " .. action.command, prefill = prefill, field = action.field, one = action.one }
end

return M
