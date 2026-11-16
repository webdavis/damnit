local diff = require("damnit.status.diff")

local Details = {}

---@param target damnit.StatusTarget
---@return string
local function key_of(target)
  return target.section .. ":" .. target.id
end

---@param targets table<integer, damnit.StatusTarget>
---@return damnit.Mark[]
function Details:detail_marks(targets)
  local marks = {}
  for line, target in pairs(targets) do
    if target.details and self.shown[key_of(target)] then
      table.insert(marks, { line = line - 1, virt_lines = diff.lines(target.details, target.named) })
    end
  end
  return marks
end

---@param target damnit.StatusTarget|nil
---@return damnit.StatusTarget[]
function Details:with_details(target)
  if target and target.kind == "section" then
    return vim.tbl_filter(function(row)
      return row.details ~= nil and row.section == target.section
    end, vim.tbl_values(self.targets))
  end
  return target and target.details and { target } or {}
end

function Details:toggle_details()
  local rows = self:with_details(self:target())
  if #rows == 0 then
    return self.app.say:warn("= shows what a change, waiting item or set changes, or every one's on its header")
  end
  local shows = vim.iter(rows):any(function(row)
    return #diff.lines(row.details, row.named) > 0
  end)
  if not shows then
    return self.app.say:info("nothing more to show for this row")
  end
  local open = vim.iter(rows):any(function(row)
    return not self.shown[key_of(row)]
  end)
  for _, row in ipairs(rows) do
    self.shown[key_of(row)] = open or nil
  end
  self:draw()
end

return Details
