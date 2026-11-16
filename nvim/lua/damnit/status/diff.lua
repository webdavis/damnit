local confirm_lines = require("damnit.kernel.confirm_lines")

local M = {}

local INDENT = "    "
local ARROW = " → "

---@param object? damnit.Object
---@param field string
---@return string
function M.value(object, field)
  if not object then
    return "none"
  end
  local value = object[field]
  if value == nil then
    value = (object.task or {})[field]
  end
  if value == nil then
    value = (object.event or {})[field]
  end
  return (confirm_lines.value(value):gsub("%s*\n%s*", " "))
end

local function rows_of(changes, named)
  local rows = {}
  for _, change in ipairs(changes) do
    if change.op == "delete" and #change.fields == 0 then
      local lead = named and (change.subject .. "  ") or ""
      table.insert(rows, { removed = ("%sdelete: the %s is removed"):format(lead, change.kind) })
    end
    for _, field in ipairs(change.fields) do
      local lead = named and { change.subject } or {}
      table.insert(rows, vim.list_extend(lead, { field, M.value(change.before, field), M.value(change.after, field) }))
    end
  end
  return rows
end

local function widths_of(rows)
  local widths = {}
  for _, row in ipairs(rows) do
    for n = 1, #row - 2 do
      widths[n] = math.max(widths[n] or 0, vim.fn.strdisplaywidth(row[n]))
    end
  end
  return widths
end

---@param changes damnit.StatusChange[] each with before and after, as dam status --full prints them
---@param named? boolean lead each line with its change's subject
---@return string[][][] one virtual line per changed field: the field, its value before and after
function M.lines(changes, named)
  local rows = rows_of(changes, named)
  local widths = widths_of(rows)
  return vim.tbl_map(function(row)
    if row.removed then
      return { { INDENT .. row.removed, "DamRemoved" } }
    end
    local lead = {}
    for n, width in ipairs(widths) do
      table.insert(lead, row[n] .. (" "):rep(width - vim.fn.strdisplaywidth(row[n])))
    end
    return {
      { INDENT .. table.concat(lead, "  ") .. "  ", "DamMeta" },
      { row[#row - 1], "DamRemoved" },
      { ARROW, "DamMeta" },
      { row[#row], "DamAdded" },
    }
  end, rows)
end

return M
