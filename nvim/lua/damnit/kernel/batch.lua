local M = {}

local POSITIONAL = { restore = true, add = true }

---@class damnit.Batch
---@field command string
---@field oids string[]
---@field flags string[]
---@field preview? boolean
---@field token? boolean

---@param batch damnit.Batch
---@return string[] args, string|nil stdin
function M.call(batch)
  local args, stdin = { batch.command }, nil
  local many = #batch.oids > 1
  if POSITIONAL[batch.command] or not many then
    vim.list_extend(args, batch.oids)
    vim.list_extend(args, batch.flags)
  else
    table.insert(args, "-")
    vim.list_extend(args, batch.flags)
    vim.list_extend(args, { "--expect", tostring(#batch.oids) })
    stdin = table.concat(batch.oids, "\n") .. "\n"
  end
  if batch.preview then
    table.insert(args, "--preview")
  end
  if batch.token and many and not POSITIONAL[batch.command] then
    table.insert(args, "--as-one")
  end
  return args, stdin
end

return M
