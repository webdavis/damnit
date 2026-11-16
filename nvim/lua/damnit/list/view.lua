local M = {}

M.TODAY = {
  { title = "Overdue", overdue = true },
  { title = "Today", due = "today", overdue = false },
  { title = "Next 7 days", due = "today+1..today+7", overdue = false },
  { title = "No date", due = "none" },
}

local NOT_FLAGS = { title = true, group = true }

---@param request table
---@return string
local function encoded(request)
  local flags = {}
  for key, value in pairs(request) do
    if not NOT_FLAGS[key] then
      flags[key] = value
    end
  end
  return vim.json.encode(next(flags) and flags or vim.empty_dict())
end

local function word(value)
  local text = tostring(value)
  if not text:find("[%s\"']") then
    return text
  end
  return '"' .. text:gsub('[\\"]', "\\%0") .. '"'
end

local function conditions(key, value)
  if key == "any" then
    local words = {}
    for _, condition in ipairs(value) do
      for field in vim.spairs(condition) do
        local each = condition[field]
        table.insert(
          words,
          "--any " .. word(field .. "=" .. (type(each) == "table" and table.concat(each, ",") or tostring(each)))
        )
      end
    end
    return words
  end
  local flag = "--" .. key:gsub("[_.]", "-")
  if value == true then
    return { flag }
  end
  return vim.tbl_map(function(each)
    return flag .. " " .. word(each)
  end, type(value) == "table" and value or { value })
end

---@param request table a view's conditions, as dam list --request reads them
---@return string the same conditions as dam list's flags
function M.flags(request)
  local words = {}
  for key, value in vim.spairs(request) do
    if not NOT_FLAGS[key] then
      vim.list_extend(words, conditions(key, value))
    end
  end
  return table.concat(words, " ")
end

---@param view string|damnit.ViewSpec|damnit.ViewSpec[]|nil
---@param views table<string, string|damnit.ViewSpec>
---@param saved table<string, table> each saved filter's table, by name
---@return string|nil flags the view's conditions as dam list's flags, nil for a view of sections
function M.conditions(view, views, saved)
  local spec = type(view) == "string" and (views[view] or view) or view
  if spec == nil then
    return ""
  end
  if type(spec) == "string" then
    return saved[spec] and M.flags(saved[spec]) or ("--filter " .. spec)
  end
  if vim.islist(spec) and #spec > 0 then
    return nil
  end
  return M.flags(spec)
end

---@class damnit.ListPart
---@field title? string
---@field args string[]
---@field stdin? string

---@param view string|damnit.ViewSpec|damnit.ViewSpec[]|nil
---@param views table<string, string|damnit.ViewSpec>
---@return damnit.ListPart[] parts, string|string[]|nil grouped_by the sort whose first key groups the rows
function M.parts(view, views)
  local spec = type(view) == "string" and (views[view] or view) or view
  if type(spec) == "table" and vim.islist(spec) and #spec > 0 then
    return vim.tbl_map(function(section)
      return { title = section.title, args = { "list", "--request", "-" }, stdin = encoded(section) }
    end, spec)
  end
  local args, stdin = M.call(view, views)
  return { { args = args, stdin = stdin } }, type(spec) == "table" and spec.group and spec.sort or nil
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

---@param view string|damnit.ViewSpec|nil
---@param views table<string, string|damnit.ViewSpec>
---@return boolean
function M.global(view, views)
  local spec = type(view) == "string" and views[view] or view
  return type(spec) == "table" and spec.global == true
end

---@param view string|damnit.ViewSpec|nil
---@param views table<string, string|damnit.ViewSpec>
---@return string|nil
function M.single_path(view, views)
  local spec = type(view) == "string" and views[view] or view
  local path = type(spec) == "table" and spec.path or nil
  if type(path) == "table" then
    path = #path == 1 and path[1] or nil
  end
  if type(path) == "string" and not path:find(",") then
    return path
  end
end

return M
