local errors = require("damnit.kernel.error")

local M = {}

local NAME = "damnit.refusals"

local function namespace()
  local id = vim.api.nvim_create_namespace(NAME)
  vim.diagnostic.config({ virtual_text = false, virtual_lines = true, signs = false }, id)
  return id
end

---@param err damnit.Error
---@param oids table<integer, string> each line's oid
---@return integer[] lines counted from 0
function M.rows_of(err, oids)
  local named = {}
  for _, oid in ipairs(err.oids or {}) do
    named[oid] = true
  end
  local lines = {}
  for line, oid in pairs(oids) do
    if named[oid] then
      table.insert(lines, line - 1)
    end
  end
  table.sort(lines)
  return #lines > 0 and lines or { 0 }
end

local function flag_named(err)
  local word = type(err.usage) == "table" and err.usage.word or nil
  return word or err.message:match("%-%-[%w-]+")
end

---@param err damnit.Error
---@param lines string[] the task buffer's lines
---@return integer line counted from 0
function M.header_line(err, lines)
  local flag = flag_named(err)
  if not flag then
    return 0
  end
  local field = flag:gsub("^%-%-", ""):gsub("^no%-", ""):gsub("^unlabels$", "labels")
  for n, line in ipairs(lines) do
    if vim.startswith(line, field .. ":") then
      return n - 1
    end
  end
  return 0
end

---@param buf integer
---@param err damnit.Error
---@param lines integer[] counted from 0
function M.show(buf, err, lines)
  local message = errors.text(err)
  vim.diagnostic.set(
    namespace(),
    buf,
    vim.tbl_map(function(line)
      return { lnum = line, col = 0, severity = vim.diagnostic.severity.ERROR, message = message, source = "dam" }
    end, lines)
  )
end

---@param buf integer
function M.clear(buf)
  vim.diagnostic.reset(namespace(), buf)
end

return M
