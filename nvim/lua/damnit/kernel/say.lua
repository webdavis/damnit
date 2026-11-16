local errors = require("damnit.kernel.error")

local Say = {}
Say.__index = Say

local M = {}

---@return damnit.Say
function M.new()
  return setmetatable({ errors = {} }, Say)
end

function Say:info(text)
  vim.notify(text, vim.log.levels.INFO, { title = "dam" })
end

function Say:warn(text)
  vim.notify(text, vim.log.levels.WARN, { title = "dam" })
end

---@param err damnit.Error
function Say:error(err)
  table.insert(self.errors, err)
  vim.notify(errors.text(err), vim.log.levels.ERROR, { title = "dam" })
end

---@return damnit.Error[]
function Say:log()
  return vim.list_slice(self.errors)
end

---@param cwd string
function Say:changed(cwd)
  vim.api.nvim_exec_autocmds("User", { pattern = "DamChanged", data = { cwd = cwd } })
end

return M
