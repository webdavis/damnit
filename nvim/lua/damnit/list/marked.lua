local M = {}

---@param marked table<string, boolean>
---@param oids string[]
---@return table<string, boolean>
function M.toggled(marked, oids)
  local next_marked = vim.deepcopy(marked)
  for _, oid in ipairs(oids) do
    next_marked[oid] = not marked[oid] or nil
  end
  return next_marked
end

---@param marked table<string, boolean>
---@param oids table<integer, string> each line's oid
---@return table<string, boolean>
function M.kept(marked, oids)
  local kept = {}
  for _, oid in pairs(oids) do
    kept[oid] = marked[oid]
  end
  return kept
end

---@param marked table<string, boolean>
---@param oids table<integer, string>
---@return damnit.Mark[]
function M.marks(marked, oids)
  local marks = {}
  for line, oid in pairs(oids) do
    if marked[oid] then
      table.insert(marks, { line = line - 1, line_group = "DamMarked" })
    end
  end
  table.sort(marks, function(a, b)
    return a.line < b.line
  end)
  return marks
end

---@param marked table<string, boolean>
---@return integer
function M.count(marked)
  return vim.tbl_count(marked)
end

return M
