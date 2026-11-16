local days = require("damnit.kernel.days")

local M = {}

local BODY_LINES = 12

---@class damnit.Card
---@field title string
---@field lines string[]
---@field marks damnit.Mark[]
---@field footer string[][]
---@field width integer

local function category_words(category)
  local words = {}
  for group, held in pairs(category or {}) do
    for _, label in ipairs(type(held) == "table" and held or { held }) do
      table.insert(words, group .. "=" .. label)
    end
  end
  table.sort(words)
  return table.concat(words, ", ")
end

local function due_text(object)
  local due = object.task and object.task.due
  local words = due and days.words(object.days_until_due)
  return words and (due .. " · " .. words) or due
end

local function priority_text(priority)
  return priority and priority < 4 and ("p" .. priority) or nil
end

---@param object damnit.Object
---@return { [1]: string, [2]: string|nil }[]
local function fields(object)
  local task, event = object.task or {}, object.event or {}
  local own = object.kind == "event"
      and { { "start", event.start }, { "end", event["end"] }, { "location", event.location } }
    or {
      { "due", due_text(object) },
      { "deadline", task.deadline },
      { "priority", priority_text(task.priority) },
      { "labels", table.concat(object.labels or {}, ", ") },
      { "category", category_words(object.category) },
      { "estimate", task.estimate and (task.estimate .. "m") },
      { "recurrence", object.recurrence },
    }
  vim.list_extend(own, { { "path", object.path }, { "parent", object.parent and object.parent:sub(1, 7) } })
  return vim.tbl_filter(function(field)
    return field[2] ~= nil and field[2] ~= ""
  end, own)
end

local function body_lines(body)
  if (body or "") == "" then
    return {}
  end
  local lines = vim.split(body, "\n", { plain = true })
  if #lines <= BODY_LINES then
    return lines
  end
  local kept = vim.list_slice(lines, 1, BODY_LINES)
  table.insert(kept, ("… %d more lines"):format(#lines - BODY_LINES))
  return kept
end

---@param object damnit.Object
---@return string[][]
local function footer(object)
  local chunks = { { " " .. object.oid:sub(1, 7), "DamMeta" } }
  for _, flag in ipairs({ { "overdue", "DamOverdue" }, { "blocked", "DamBlocked" } }) do
    if object[flag[1]] then
      vim.list_extend(chunks, { { " · " }, flag })
    end
  end
  table.insert(chunks, { " " })
  return chunks
end

local function widest(texts)
  return math.max(0, unpack(vim.tbl_map(vim.fn.strdisplaywidth, texts)))
end

---@param object damnit.Object
---@param room integer the widest the card may be
---@return damnit.Card
function M.render(object, room)
  local shown = fields(object)
  local name_width = widest(vim.tbl_map(function(field)
    return field[1]
  end, shown)) + 2
  local lines, marks = {}, {}
  for n, field in ipairs(shown) do
    table.insert(lines, field[1] .. (" "):rep(name_width - #field[1]) .. field[2])
    table.insert(marks, { line = n - 1, col = 0, end_col = #field[1], group = "DamMeta" })
  end
  local body = body_lines(object.body)
  local chunks = footer(object)
  local footer_text = table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, chunks))
  local width = math.min(
    room,
    math.max(widest(lines), widest(body), vim.fn.strdisplaywidth(object.subject) + 2, widest({ footer_text }))
  )
  if #body > 0 then
    table.insert(lines, ("─"):rep(width))
    vim.list_extend(lines, body)
  end
  return { title = object.subject, lines = lines, marks = marks, footer = chunks, width = width }
end

return M
