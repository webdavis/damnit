local M = {}

---@param lines string[]
---@param label string
---@param value string|nil
local function field(lines, label, value)
  if value and value ~= "" then
    table.insert(lines, label .. ": " .. value)
  end
end

---@param lines string[]
---@param text string|nil
local function paragraph(lines, text)
  if text and text ~= "" then
    vim.list_extend(lines, { "", text })
  end
end

---@param object damnit.Object
---@param note string|nil
---@return string
function M.text(object, note)
  local lines = {}
  field(lines, "task", object.subject)
  field(lines, "oid", object.oid:sub(1, 7))
  field(lines, "path", object.path)
  field(lines, "labels", table.concat(object.labels or {}, ", "))
  field(lines, "due", object.task and object.task.due)
  paragraph(lines, object.body)
  paragraph(lines, note and note ~= "" and ("note: " .. note) or nil)
  paragraph(lines, ("dam show %s -o json"):format(object.oid))
  return table.concat(lines, "\n")
end

return M
