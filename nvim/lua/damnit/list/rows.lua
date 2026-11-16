local M = {}

local FLAGS = {
  {
    group = "DamDone",
    holds = function(object)
      return object.task ~= nil and object.task.done
    end,
  },
  {
    group = "DamOverdue",
    sign = "overdue",
    holds = function(object)
      return object.overdue
    end,
  },
  {
    group = "DamBlocked",
    sign = "blocked",
    holds = function(object)
      return object.blocked
    end,
  },
}

---@param object damnit.Object
---@return string text, integer|nil priority_end the end of the priority word, when shown
local function text_of(object)
  local task = object.task
  if not task then
    return object.subject, nil
  end
  local box = task.done and "[x] " or "[ ] "
  if task.priority and task.priority < 4 then
    local priority = "p" .. task.priority
    return box .. priority .. " " .. object.subject, #box + #priority
  end
  return box .. object.subject, nil
end

---@param rows damnit.Row[]
---@param signs { overdue: string, blocked: string }
---@return string[] lines, damnit.Mark[] marks
function M.render(rows, signs)
  local lines, marks = {}, {}
  for n, row in ipairs(rows) do
    local object = row.object
    local indent = string.rep("  ", row.depth)
    local text, priority_end = text_of(object)
    lines[n] = indent .. text
    for _, flag in ipairs(FLAGS) do
      if flag.holds(object) then
        table.insert(marks, { line = n - 1, col = #indent, end_col = #lines[n], group = flag.group })
        if flag.sign then
          table.insert(marks, { line = n - 1, sign = signs[flag.sign], sign_group = flag.group })
        end
      end
    end
    if priority_end then
      local col = #indent + priority_end - 2
      table.insert(marks, { line = n - 1, col = col, end_col = col + 2, group = "DamPriority" })
    end
  end
  return lines, marks
end

return M
