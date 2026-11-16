local M = {}

local ALL_DAY = "all day    "
local SEPARATOR = { " · " }

---@class damnit.AgendaEvent
---@field oid string
---@field subject string
---@field path string
---@field all_day boolean
---@field start string
---@field end string
---@field busy boolean
---@field day string
---@field weekday string

---@class damnit.AgendaAnswer
---@field from string
---@field to string
---@field events damnit.AgendaEvent[]
---@field tasks? damnit.Object[]

---@param event damnit.AgendaEvent
---@return string
local function time_of(event)
  if event.all_day then
    return ALL_DAY
  end
  return event.start:sub(12, 16) .. "–" .. event["end"]:sub(12, 16)
end

---@param answer damnit.AgendaAnswer
---@return damnit.Segment[]
function M.segments(answer)
  local order, by_day = {}, {}
  local function day_of(row)
    if not by_day[row.day] then
      by_day[row.day] = { title = row.weekday .. " " .. row.day, uncounted = true, rows = {}, tasks = {} }
      table.insert(order, row.day)
    end
    return by_day[row.day]
  end
  for _, event in ipairs(answer.events) do
    table.insert(day_of(event).rows, { object = event, depth = 0, lead = time_of(event), readonly = true })
  end
  for _, task in ipairs(answer.tasks or {}) do
    table.insert(day_of(task).tasks, { object = task, depth = 0 })
  end
  table.sort(order)
  return vim.tbl_map(function(day)
    local segment = by_day[day]
    vim.list_extend(segment.rows, segment.tasks)
    segment.tasks = nil
    return segment
  end, order)
end

local function counted(count, one, many)
  return ("%d %s"):format(count, count == 1 and one or many)
end

---@param answer damnit.AgendaAnswer
---@param marked integer
---@return string[][]
function M.bar(answer, marked)
  local parts = {
    { " agenda", "DamHeader" },
    { answer.from .. " … " .. answer.to },
    { counted(#answer.events, "event", "events") },
    { #(answer.tasks or {}) .. " due", "DamDue" },
  }
  if marked > 0 then
    table.insert(parts, { marked .. " marked", "DamMarked" })
  end
  local chunks = {}
  for n, part in ipairs(parts) do
    table.insert(chunks, n > 1 and SEPARATOR or nil)
    table.insert(chunks, part)
  end
  return chunks
end

---@param unit "day"|"week"
---@param offset integer
---@return string
function M.window(unit, offset)
  local word = unit == "day" and "today" or "week"
  if offset == 0 then
    return word
  end
  return ("%s%+d"):format(word, offset)
end

return M
