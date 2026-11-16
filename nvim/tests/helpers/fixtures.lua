local M = {}

local fixtures_dir = vim.fs.joinpath(vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2))), "fixtures")

---@param name string
---@return string
function M.read(name)
  local file = assert(io.open(vim.fs.joinpath(fixtures_dir, name), "r"))
  local text = file:read("*a")
  file:close()
  return text
end

---@param name string
function M.answer(name)
  return { code = 0, stdout = M.read(name), stderr = "" }
end

---@param name string
---@param code integer
function M.refusal(name, code)
  return { code = code, stdout = "", stderr = M.read(name) }
end

return M
