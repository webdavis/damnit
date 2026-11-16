local M = {}

---@param days any dam's whole days until a date, negative when past
---@return string|nil
function M.words(days)
  if type(days) ~= "number" then
    return nil
  end
  if days < 0 then
    return -days .. "d late"
  end
  return ({ [0] = "today", [1] = "tomorrow" })[days] or ("in " .. days .. "d")
end

return M
