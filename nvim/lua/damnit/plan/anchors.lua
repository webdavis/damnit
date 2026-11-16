local M = {}

M.LIMIT = 50
M.UNPLANNED = "Unplanned"

local DAYS = 6

---@param offset integer
---@return string
local function word(offset)
  return offset == 0 and "today" or ("today+" .. offset)
end

---@return damnit.ViewSpec[]
function M.sections()
  local sections = { { title = M.UNPLANNED, ["focus.day"] = "none", blocked = false, limit = M.LIMIT } }
  for offset = 0, DAYS do
    table.insert(sections, { title = word(offset), ["focus.day"] = word(offset) })
  end
  return sections
end

---@param title string
---@param objects damnit.Object[]
---@return string
function M.header(title, objects)
  if title == M.UNPLANNED and #objects >= M.LIMIT then
    return ("%s (first %d)"):format(title, M.LIMIT)
  end
  local first = objects[1]
  local focus = first and first.task and first.task.focus
  if focus then
    return ("%s · %s (%d)"):format(title, focus.day, #objects)
  end
  return ("%s (%d)"):format(title, #objects)
end

---@param title string
---@return integer|nil offset the day's distance from today; nil for the unplanned section
local function offset_of(title)
  if title == "today" then
    return 0
  end
  return tonumber(title:match("^today%+(%d+)$"))
end

---@param title string the section the rows sit in
---@param step 1|-1
---@return string|nil flags, string|nil problem
function M.moved(title, step)
  local offset = offset_of(title)
  if not offset then
    if step < 0 then
      return nil, "< moves a planned task a day earlier; this one is not planned"
    end
    return "--focus today", nil
  end
  if offset + step < 0 then
    return "--no-focus", nil
  end
  return "--focus " .. word(offset + step), nil
end

---@param rows damnit.Row[]
---@param oids string[]
---@return string|nil title the one section holding every oid
function M.section_of(rows, oids)
  local wanted, found, title = {}, nil, nil
  for _, oid in ipairs(oids) do
    wanted[oid] = true
  end
  for _, row in ipairs(rows) do
    title = row.header and row.key or title
    if row.object and wanted[row.object.oid] then
      if found and found ~= title then
        return nil
      end
      found = title
    end
  end
  return found
end

return M
