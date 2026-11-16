local confirm_lines = require("damnit.kernel.confirm_lines")

local M = {}

local function categories(category)
  local words = {}
  for group, held in pairs(category or {}) do
    for _, label in ipairs(type(held) == "table" and held or { held }) do
      table.insert(words, group .. "=" .. label)
    end
  end
  table.sort(words)
  return words
end

local READ = {
  subject = function(object)
    return object.subject
  end,
  path = function(object)
    return object.path
  end,
  labels = function(object)
    return object.labels
  end,
  category = function(object)
    return categories(object.category)
  end,
  recurrence = function(object)
    return object.recurrence
  end,
}

local function before(object, field)
  local read = READ[field]
  return confirm_lines.value(read and read(object) or (object.task or object.event or {})[field])
end

local function after(change, field)
  local read = READ[field]
  return confirm_lines.value(read and read(change) or change[field])
end

local function changes_by_oid(answer)
  local found = {}
  for _, change in ipairs(answer.changes) do
    found[change.oid] = change
  end
  return found
end

local function cells(answer, objects, field)
  local changes, rows = changes_by_oid(answer), {}
  for _, oid in ipairs(answer.oids) do
    local object, change = objects[oid] or { subject = oid:sub(1, 7) }, changes[oid]
    for _, name in ipairs(change and (field and { field } or change.fields) or {}) do
      table.insert(rows, { object.subject, name, before(object, name) .. " → " .. after(change, name) })
    end
    if not change then
      table.insert(rows, { object.subject, "unchanged" })
    end
  end
  return rows
end

local function joined(rows, from)
  local width = 0
  for _, row in ipairs(rows) do
    width = math.max(width, vim.fn.strdisplaywidth(row[from]))
  end
  return vim.tbl_map(function(row)
    local first = row[from] .. (" "):rep(width - vim.fn.strdisplaywidth(row[from]))
    return vim.trim(table.concat({ first, unpack(row, from + 1) }, "  "))
  end, rows)
end

---@param answer { oids: string[], changes: table[] } a --preview answer
---@param objects table<string, damnit.Object> the rows on screen by oid
---@param field? string the field the key changes; nil for every field dam lists
---@return string[]
function M.rows(answer, objects, field)
  return joined(cells(answer, objects, field), 1)
end

---@param answer { oids: string[], changes: table[] }
---@param objects table<string, damnit.Object>
---@param field? string
---@return string
function M.reading(answer, objects, field)
  if #answer.changes == 0 then
    return "no change"
  end
  local rows = cells(answer, objects, field)
  return table.concat(joined(rows, 2), " · ")
end

return M
