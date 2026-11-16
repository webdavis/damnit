local M = {}

local SEPARATOR = { " · " }

local function counted(count, one, many)
  return ("%d %s"):format(count, count == 1 and one or many)
end

local function tally(objects)
  local counts = { task = 0, event = 0, overdue = 0, blocked = 0 }
  for _, object in ipairs(objects) do
    counts[object.kind] = (counts[object.kind] or 0) + 1
    counts.overdue = counts.overdue + (object.overdue and 1 or 0)
    counts.blocked = counts.blocked + (object.blocked and 1 or 0)
  end
  return counts
end

---@param view string
---@param path string
---@param objects damnit.Object[]
---@return string[][]
function M.chunks(view, path, objects)
  local counts = tally(objects)
  local parts = { { " " .. view, "DamHeader" }, { path }, { counted(counts.task, "task", "tasks") } }
  local optional = {
    { counts.event, counted(counts.event, "event", "events") },
    { counts.overdue, counts.overdue .. " overdue", "DamOverdue" },
    { counts.blocked, counts.blocked .. " blocked", "DamBlocked" },
  }
  for _, part in ipairs(optional) do
    if part[1] > 0 then
      table.insert(parts, { part[2], part[3] })
    end
  end
  local chunks = {}
  for n, part in ipairs(parts) do
    if n > 1 then
      table.insert(chunks, SEPARATOR)
    end
    table.insert(chunks, part)
  end
  table.insert(chunks, 4, { "", cut = true })
  return chunks
end

return M
