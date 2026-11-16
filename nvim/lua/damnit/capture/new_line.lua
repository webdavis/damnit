local argv = require("damnit.kernel.argv")

local M = {}

---@param text string
---@param flags damnit.Flag[] dam new's flags
---@return string[]
function M.args(text, flags)
  local subject, rest = argv.subject(argv.args(text, flags))
  local args = { "new" }
  if subject ~= "" then
    table.insert(args, subject)
  end
  return vim.list_extend(args, rest)
end

---@param answer { changes: table[] }
---@return string
function M.reading(answer)
  local change = answer.changes[1]
  local parts = { ("%s %q"):format(change.kind, change.subject) }
  if change.due then
    table.insert(parts, "due " .. change.due)
  end
  if #(change.labels or {}) > 0 then
    table.insert(parts, table.concat(change.labels, ","))
  end
  if (change.path or "") ~= "" then
    table.insert(parts, change.path)
  end
  return table.concat(parts, " · ")
end

return M
