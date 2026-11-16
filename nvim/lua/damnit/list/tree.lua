local M = {}

---@class damnit.Row
---@field object damnit.Object
---@field depth integer

---@param objects damnit.Object[]
---@return table<string, damnit.Object[]> children, damnit.Object[] roots
local function family(objects)
  local listed = {}
  for _, object in ipairs(objects) do
    listed[object.oid] = true
  end
  local children, roots = {}, {}
  for _, object in ipairs(objects) do
    if object.parent and listed[object.parent] then
      children[object.parent] = children[object.parent] or {}
      table.insert(children[object.parent], object)
    else
      table.insert(roots, object)
    end
  end
  return children, roots
end

---@param objects damnit.Object[]
---@return damnit.Row[]
function M.arranged(objects)
  local children, roots = family(objects)
  local rows, placed = {}, {}
  local function place(object, depth)
    if placed[object] then
      return
    end
    placed[object] = true
    table.insert(rows, { object = object, depth = depth })
    for _, child in ipairs(children[object.oid] or {}) do
      place(child, depth + 1)
    end
  end
  for _, root in ipairs(roots) do
    place(root, 0)
  end
  for _, object in ipairs(objects) do
    place(object, 0)
  end
  return rows
end

return M
