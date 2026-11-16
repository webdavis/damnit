local M = {}

---@param message string
---@return damnit.Error
function M.plugin(message)
  return { kind = "plugin", message = message, oids = {} }
end

---@param dam string
---@return damnit.Error
function M.missing(dam)
  return M.plugin(('cannot run %q; install dam or set setup({ dam = "/path/to/dam" })'):format(dam))
end

local function document_of(stderr)
  local ok, decoded = pcall(vim.json.decode, stderr or "", { luanil = { object = true, array = true } })
  if ok and type(decoded) == "table" and type(decoded.error) == "table" and type(decoded.error.message) == "string" then
    return decoded.error
  end
end

---@param code integer
---@param stderr string
---@return damnit.Error
function M.of_exit(code, stderr)
  local text = vim.trim(stderr or "")
  return document_of(text) or M.plugin(text ~= "" and text or ("dam exited %d"):format(code))
end

---@param stderr string
---@return damnit.Error
function M.not_json(stderr)
  local text = vim.trim(stderr or "")
  return M.plugin("dam's answer was not JSON" .. (text ~= "" and (": " .. text) or ""))
end

---@param err damnit.Error
---@return string|nil
function M.suggested(err)
  local command = type(err.selection) == "table" and err.selection.command or nil
  if command and not err.message:find(command, 1, true) then
    return command
  end
end

---@param err damnit.Error
---@return string
function M.text(err)
  local command = M.suggested(err)
  return command and ("%s\nrun: %s"):format(err.message, command) or err.message
end

return M
