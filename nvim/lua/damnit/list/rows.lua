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
    holds = function(object)
      return object.overdue
    end,
  },
  {
    group = "DamBlocked",
    holds = function(object)
      return object.blocked
    end,
  },
}

local function category_words(category)
  local words = {}
  for group, label in pairs(category or {}) do
    table.insert(words, group .. "=" .. label)
  end
  table.sort(words)
  return words
end

---@param object damnit.Object
---@return string
function M.text(object)
  local task = object.task
  local words = {}
  if task then
    table.insert(words, task.done and "[x]" or "[ ]")
    if task.priority and task.priority < 4 then
      table.insert(words, "p" .. task.priority)
    end
  end
  table.insert(words, object.subject)
  table.insert(words, task and task.due or object.event and object.event.start)
  local in_category = {}
  for _, label in pairs(object.category or {}) do
    in_category[label] = true
  end
  for _, label in ipairs(object.labels or {}) do
    if not in_category[label] then
      table.insert(words, "#" .. label)
    end
  end
  vim.list_extend(words, category_words(object.category))
  return table.concat(words, " ")
end

---@param rows damnit.Row[]
---@return string[] lines, damnit.Mark[] marks
function M.render(rows)
  local lines, marks = {}, {}
  for n, row in ipairs(rows) do
    local object = row.object
    local indent = string.rep("  ", row.depth)
    lines[n] = indent .. M.text(object)
    for _, flag in ipairs(FLAGS) do
      if flag.holds(object) then
        table.insert(marks, { line = n - 1, col = #indent, end_col = #lines[n], group = flag.group })
      end
    end
  end
  return lines, marks
end

return M
