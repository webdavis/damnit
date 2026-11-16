local M = {}

---@class damnit.Pick
---@field row integer the cursor's line
---@field count integer the count typed before the key, 0 for none
---@field range? integer[] the visual selection's first and last line, either order
---@field marked table<string, boolean>
---@field lines integer

---@param oids table<integer, string> each line's oid; a header line has none
---@param from integer
---@param to integer
---@param keep fun(oid: string): boolean
---@return string[]
local function between(oids, from, to, keep)
  local found = {}
  for line = from, to do
    local oid = oids[line]
    if oid and keep(oid) then
      table.insert(found, oid)
    end
  end
  return found
end

local function any()
  return true
end

---@param oids table<integer, string>
---@param pick damnit.Pick
---@return string[] oids in screen order
function M.oids(oids, pick)
  if pick.range then
    return between(oids, math.min(unpack(pick.range)), math.max(unpack(pick.range)), any)
  end
  if next(pick.marked) then
    return between(oids, 1, pick.lines, function(oid)
      return pick.marked[oid] == true
    end)
  end
  local last = pick.count > 0 and math.min(pick.row + pick.count - 1, pick.lines) or pick.row
  return between(oids, pick.row, last, any)
end

return M
