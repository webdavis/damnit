local Browse = {}
Browse.__index = Browse

local M = {}

---@class damnit.HistoryBrowse
---@field older fun(self, current: string): string|nil
---@field newer fun(self): string|nil

---@param entries string[]
---@return damnit.HistoryBrowse
function M.browse(entries)
  return setmetatable({ entries = entries, index = #entries + 1 }, Browse)
end

---@param entries string[]
---@param text string
function M.kept(entries, text)
  if entries[#entries] ~= text then
    table.insert(entries, text)
  end
end

function Browse:older(current)
  if self.index <= 1 then
    return nil
  end
  if self.index > #self.entries then
    self.draft = current
  end
  self.index = self.index - 1
  return self.entries[self.index]
end

function Browse:newer()
  if self.index > #self.entries then
    return nil
  end
  self.index = self.index + 1
  return self.entries[self.index] or self.draft
end

return M
