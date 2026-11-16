local M = {}

---@param request table
---@return string
local function encoded(request)
  return vim.json.encode(next(request) and request or vim.empty_dict())
end

---@param view string|damnit.ViewSpec|nil
---@param views table<string, string|damnit.ViewSpec>
---@return string[] args, string|nil stdin
function M.call(view, views)
  if view == nil then
    return { "list" }, nil
  end
  local spec = type(view) == "string" and (views[view] or view) or view
  if type(spec) == "string" then
    return { "list", "--filter", spec }, nil
  end
  return { "list", "--request", "-" }, encoded(spec)
end

return M
