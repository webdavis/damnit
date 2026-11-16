local M = {}

local GAP = { "  " }
local ELLIPSIS = "…"

local function width_of(chunks)
  local width = 0
  for _, chunk in ipairs(chunks) do
    width = width + vim.fn.strdisplaywidth(chunk[1])
  end
  return width
end

local function first_value(chunks)
  for n, chunk in ipairs(chunks) do
    if vim.trim(chunk[1]) ~= "" then
      return n
    end
  end
end

---@param line integer
---@param text_width integer the display width of the row's text
---@param chunks string[][] the columns, each value after a two-cell gap
---@param width integer the window's text width
---@return damnit.Mark|nil
function M.right(line, text_width, chunks, width)
  local first = first_value(chunks)
  if not first then
    return text_width > width and { line = line, virt = { { ELLIPSIS, "DamMeta" } }, align = "right_align" } or nil
  end
  local virt = vim.list_extend({ GAP }, chunks, first)
  if text_width + width_of(virt) > width then
    virt[1] = { ELLIPSIS .. " ", "DamMeta" }
  end
  return { line = line, virt = virt, align = "right_align" }
end

return M
