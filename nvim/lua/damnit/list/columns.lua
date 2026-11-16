local M = {}

local ORDER = { "labels", "category", "waits", "estimate", "deadline", "days", "due" }
local DROPPED_FIRST = { "labels", "category", "waits", "estimate", "deadline", "days" }
local GAP = "  "

M.gap = #GAP

---@alias damnit.Cell { [1]: string, [2]: string }

---@class damnit.Layout
---@field cells table<string, damnit.Cell>[]
---@field widths table<string, integer>

local function values_of(held)
  return type(held) == "table" and held or { held }
end

local function in_category(object)
  local words, taken = {}, {}
  for group, held in pairs(object.category or {}) do
    for _, label in ipairs(values_of(held)) do
      table.insert(words, group .. "=" .. label)
      taken[label] = true
    end
  end
  table.sort(words)
  return words, taken
end

local function days_words(days)
  if days == nil then
    return nil
  end
  if days < 0 then
    return -days .. "d late"
  end
  return ({ [0] = "today", [1] = "tomorrow" })[days] or ("in " .. days .. "d")
end

local function waits_on(object, subjects)
  if #(object.depends or {}) == 0 then
    return nil
  end
  return "waits on "
    .. table.concat(
      vim.tbl_map(function(oid)
        return subjects[oid] or oid:sub(1, 7)
      end, object.depends),
      ", "
    )
end

local function cells_of(object, subjects)
  local task = object.task or {}
  local category, taken = in_category(object)
  local labels = vim.tbl_map(
    function(label)
      return "#" .. label
    end,
    vim.tbl_filter(function(label)
      return not taken[label]
    end, object.labels or {})
  )
  local due = task.due or (object.event and object.event.start)
  return {
    labels = { table.concat(labels, " "), "DamLabel" },
    category = { table.concat(category, " "), "DamCategory" },
    waits = { waits_on(object, subjects), "DamMeta" },
    estimate = { task.estimate and (task.estimate .. "m"), "DamMeta" },
    deadline = { task.deadline and ("deadline " .. task.deadline), "DamDue" },
    days = { days_words(task.days_until_due), "DamMeta" },
    due = { due, object.overdue and "DamOverdue" or "DamDue" },
  }
end

---@param rows damnit.Row[]
---@return damnit.Layout
function M.layout(rows)
  local subjects = {}
  for _, row in ipairs(rows) do
    subjects[row.object.oid] = row.object.subject
  end
  local layout = { cells = {}, widths = {} }
  for _, name in ipairs(ORDER) do
    layout.widths[name] = 0
  end
  for n, row in ipairs(rows) do
    layout.cells[n] = cells_of(row.object, subjects)
    for name, cell in pairs(layout.cells[n]) do
      layout.widths[name] = math.max(layout.widths[name], vim.fn.strdisplaywidth(cell[1] or ""))
    end
  end
  return layout
end

local function kept_of(widths, dropped)
  return vim.tbl_filter(function(name)
    return widths[name] > 0 and not dropped[name]
  end, ORDER)
end

local function width_of(widths, kept)
  local total = widths.text
  for _, name in ipairs(kept) do
    total = total + #GAP + widths[name]
  end
  return total
end

---@param widths table<string, integer> each column's width and `text`, the widest row text
---@param width integer the window's text width
---@return string[] kept the columns that fit, in screen order
function M.fit(widths, width)
  local dropped = {}
  for _, name in ipairs(DROPPED_FIRST) do
    if width_of(widths, kept_of(widths, dropped)) <= width then
      break
    end
    dropped[name] = true
  end
  return kept_of(widths, dropped)
end

---@param layout damnit.Layout
---@param n integer
---@param kept string[]
---@return string[][]
function M.chunks(layout, n, kept)
  local chunks = {}
  for _, name in ipairs(kept) do
    local text, group = unpack(layout.cells[n][name])
    text = text or ""
    table.insert(chunks, { GAP })
    local padded = text .. (" "):rep(layout.widths[name] - vim.fn.strdisplaywidth(text))
    table.insert(chunks, text == "" and { padded } or { padded, group })
  end
  return chunks
end

return M
