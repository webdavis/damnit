local align = require("damnit.kernel.align")

local M = {}

local GAP = "  "
local INDENT = "    "
local DIMMED = { true, true, false, true }
local OP_GROUPS = { create = "DamAdded", update = "DamChanged", delete = "DamRemoved" }

---@param at string
---@return string
local function to_the_minute(at)
  return (at:gsub("^(%d%d%d%d%-%d%d%-%d%d)T(%d%d:%d%d)[^Z]*(Z?)$", "%1 %2%3"))
end

---@param count integer
---@return string
local function counted(count)
  return ("%d change%s"):format(count, count == 1 and "" or "s")
end

---@param changes damnit.StatusChange[]
---@return string
local function subjects(changes)
  return table.concat(
    vim.tbl_map(function(change)
      return change.subject
    end, changes),
    ", "
  )
end

---@param rows string[][]
---@return integer[] the widest cell of each column but the last
local function widths_of(rows)
  local widths = {}
  for _, row in ipairs(rows) do
    for n = 1, #row - 1 do
      widths[n] = math.max(widths[n] or 0, vim.fn.strdisplaywidth(row[n]))
    end
  end
  return widths
end

local function padded(text, width)
  return text .. (" "):rep(width - vim.fn.strdisplaywidth(text))
end

---@param commits damnit.Commit[] in dam's order, newest first
---@param width integer the window's text width
---@return string[] lines, damnit.Mark[] marks
function M.render(commits, width)
  local rows = vim.tbl_map(function(commit)
    local changes = commit.changes
    return { commit.id:sub(1, 7), to_the_minute(commit.at), commit.message, counted(#changes), subjects(changes) }
  end, commits)
  local widths = widths_of(rows)
  local lines, marks = {}, {}
  for n, row in ipairs(rows) do
    local col, cells = 0, {}
    for c, cell in ipairs(row) do
      if DIMMED[c] then
        table.insert(marks, { line = n - 1, col = col, end_col = col + #cell, group = "DamMeta" })
      end
      cells[c] = widths[c] and padded(cell, widths[c]) or cell
      col = col + #cells[c] + #GAP
    end
    lines[n] = table.concat(cells, GAP)
    local cut = align.right(n - 1, vim.fn.strdisplaywidth(lines[n]), {}, width)
    if cut then
      table.insert(marks, cut)
    end
  end
  return lines, marks
end

---@param commit damnit.Commit as dam show prints a commit
---@return string[][][] one virtual line per change: its op, subject and the fields it changed
function M.changes(commit)
  local rows = vim.tbl_map(function(change)
    return { change.op, change.subject, table.concat(change.fields, ", ") }
  end, commit.changes)
  local widths = widths_of(rows)
  return vim.tbl_map(function(row)
    return {
      { INDENT },
      { padded(row[1], widths[1]), OP_GROUPS[row[1]] },
      { GAP .. padded(row[2], widths[2]) .. GAP },
      { row[3], "DamMeta" },
    }
  end, rows)
end

return M
