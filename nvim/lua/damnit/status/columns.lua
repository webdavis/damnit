local M = {}

local GAP = "  "

---@class damnit.StatusRow
---@field shape? string
---@field left string[][]
---@field right? string[][]
---@field target damnit.StatusTarget
---@field group? string

local UNITS = { { 86400, "d" }, { 3600, "h" }, { 60, "m" } }

---@param seconds integer
---@return string
function M.ago(seconds)
  for _, unit in ipairs(UNITS) do
    if seconds >= unit[1] then
      return math.floor(seconds / unit[1]) .. unit[2] .. " ago"
    end
  end
  return seconds .. "s ago"
end

local function widths_of(rows, side)
  local widths = {}
  for _, row in ipairs(rows) do
    local shape = row.shape or ""
    widths[shape] = widths[shape] or {}
    for n, cell in ipairs(row[side] or {}) do
      widths[shape][n] = math.max(widths[shape][n] or 0, vim.fn.strdisplaywidth(cell[1]))
    end
  end
  return widths
end

local function padded(text, width)
  return text .. (" "):rep(width - vim.fn.strdisplaywidth(text))
end

local SHORTEST_SUBJECT = 16
local SUBJECT_ROOM = #"  " + #GAP + SHORTEST_SUBJECT

local function fitted(right, left, width)
  for shape, widths in pairs(right) do
    local room = width - SUBJECT_ROOM - (left[shape][1] or 0)
    local block = 0
    for _, column in ipairs(widths) do
      block = block + #GAP + column
    end
    while #widths > 0 and block > room do
      block = block - #GAP - table.remove(widths)
    end
  end
  return right
end

---@param rows damnit.StatusRow[]
---@param width integer the window's text width
---@return fun(row: damnit.StatusRow): string, string[][] lay the row's text and its right-hand chunks
function M.laid(rows, width)
  local left = widths_of(rows, "left")
  local right = fitted(widths_of(rows, "right"), left, width)
  return function(row)
    local shape = row.shape or ""
    local words = {}
    for n, cell in ipairs(row.left) do
      words[n] = n < #row.left and padded(cell[1], left[shape][n]) or cell[1]
    end
    local chunks = {}
    for n, column in ipairs(right[shape] or {}) do
      local cell = row.right[n] or { "" }
      if column > 0 then
        vim.list_extend(chunks, { { GAP }, { padded(cell[1], column), cell[1] ~= "" and cell[2] or nil } })
      end
    end
    return (table.concat(words, GAP):gsub("%s+$", "")), chunks
  end
end

return M
