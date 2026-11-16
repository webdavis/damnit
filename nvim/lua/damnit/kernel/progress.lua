local M = {}

---@class damnit.ProgressRecord
---@field step string
---@field stage "started"|"finished"|"failed"
---@field done integer
---@field total? integer
---@field seconds integer
---@field remote? string

local STATUS = { started = "running", finished = "success", failed = "failed" }

---@param line string
---@return damnit.ProgressRecord|nil
function M.record(line)
  local ok, decoded = pcall(vim.json.decode, line, { luanil = { object = true, array = true } })
  if ok and type(decoded) == "table" and type(decoded.step) == "string" and STATUS[decoded.stage] then
    return decoded
  end
end

---@param record damnit.ProgressRecord
---@return string text, vim.api.keyset.echo_opts opts
function M.message(record)
  local total = record.total or 0
  return record.step,
    {
      kind = "progress",
      status = STATUS[record.stage],
      percent = total > 0 and math.floor(record.done * 100 / total) or nil,
      title = record.remote,
      source = "dam",
    }
end

return M
