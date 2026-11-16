local M = {}

local DATE_KEYS =
  { due = true, deadline = true, start = true, ["end"] = true, completed_at = true, ["focus.day"] = true }

---@param sort string|string[]|nil
---@return string|nil the first sort key, which groups the rows
function M.key(sort)
  local first = type(sort) == "table" and sort[1] or sort
  return first and vim.trim(vim.split(first, ",")[1]):gsub(":desc$", "") or nil
end

---@param object damnit.Object
---@param key string
---@return any
local function value_of(object, key)
  for _, holder in ipairs({ object, object.task or {}, object.event or {} }) do
    local value = vim.tbl_get(holder, unpack(vim.split(key, ".", { plain = true })))
    if value ~= nil then
      return value
    end
  end
end

---@param object damnit.Object
---@param key string
---@return string
local function title_of(object, key)
  local value = value_of(object, key)
  if type(value) == "table" then
    value = table.concat(value, ", ")
  end
  value = value ~= nil and tostring(value) or ""
  if DATE_KEYS[key] then
    value = value:sub(1, 10)
  end
  return value == "" and "none" or value
end

---@param objects damnit.Object[]
---@param sort string|string[]|nil the view's sort, as sent to dam
---@return damnit.Segment[]|nil
function M.segments(objects, sort)
  local key = M.key(sort)
  if not key then
    return nil
  end
  local segments = {}
  for _, object in ipairs(objects) do
    local title = title_of(object, key)
    local last = segments[#segments]
    if not last or last.title ~= title then
      last = { title = title, objects = {} }
      table.insert(segments, last)
    end
    table.insert(last.objects, object)
  end
  return segments
end

return M
