local align = require("damnit.kernel.align")
local columns = require("damnit.list.columns")
local rows = require("damnit.list.rows")

local M = {}

---@param arranged damnit.Row[]
---@param signs { overdue: string, blocked: string }
---@param width integer the narrowest text width of the windows showing the list
---@param without_path? boolean leave the path column out
---@return string[] lines, damnit.Mark[] marks
function M.render(arranged, signs, width, without_path)
  local lines, marks = rows.render(arranged, signs)
  local layout = columns.layout(arranged, without_path)
  local widest, room = nil, width - columns.gap - layout.widths.due
  for _, line in ipairs(lines) do
    local text = vim.fn.strdisplaywidth(line)
    if text <= room then
      widest = math.max(widest or 0, text)
    end
  end
  widest = widest or width
  local kept = columns.fit(vim.tbl_extend("force", layout.widths, { text = widest }), width)
  for n, line in ipairs(lines) do
    if arranged[n].object then
      table.insert(marks, align.right(n - 1, vim.fn.strdisplaywidth(line), columns.chunks(layout, n, kept), width))
    end
  end
  return lines, marks
end

return M
