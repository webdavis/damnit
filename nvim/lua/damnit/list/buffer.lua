local project = require("damnit.list.project")
local tree = require("damnit.list.tree")
local view = require("damnit.list.view")

local Lists = vim.tbl_extend(
  "error",
  {},
  require("damnit.list.drawing"),
  require("damnit.list.bindings"),
  require("damnit.list.landing")
)
Lists.__index = Lists

local M = {}

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
---@param more_keys? fun(buf: integer): damnit.KeySpec[]
function M.new(app, open_task, more_keys)
  local lists = setmetatable({ app = app, open_task = open_task, shown = {}, project = project.new(app) }, Lists)
  lists.more_keys = more_keys
  lists.group = vim.api.nvim_create_augroup("damnit.list", { clear = true })
  vim.api.nvim_create_autocmd("User", {
    group = lists.group,
    pattern = "DamChanged",
    callback = function()
      lists:changed()
    end,
  })
  vim.api.nvim_create_autocmd("WinResized", {
    group = lists.group,
    callback = function()
      for buf in pairs(lists.shown) do
        lists:refit(buf)
      end
    end,
  })
  return lists
end

---@param buf integer
function Lists:read(buf)
  local list = self.shown[buf]
  self.app.dam:run(list.call, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    if not vim.api.nvim_buf_is_valid(buf) then
      return
    end
    list.arranged = tree.arranged(answer.objects)
    list.oids = vim.tbl_map(function(row)
      return row.object.oid
    end, list.arranged)
    self:draw(buf)
    self:refit(buf)
    self:drawn(list)
  end)
end

function Lists:changed()
  for buf in pairs(self.shown) do
    self:read(buf)
  end
end

---@param name string|nil
---@param cwd string
---@param smods? table
function Lists:open(name, cwd, smods)
  local spec = name or self.app.options.default_view
  local title = "dam://list/" .. (type(spec) == "string" and spec or "default")
  local args, stdin = view.call(spec, self.app.options.views)
  local buf = self.app.screen:scratch(title, "damlist")
  if not self.shown[buf] then
    self:bind(buf)
  end
  local list = { name = title:sub(#"dam://list/" + 1), call = { args = args, stdin = stdin, cwd = cwd, key = title } }
  list.oids, list.view, list.waiting = {}, spec, {}
  self.shown[buf] = list
  self.app.screen:show(buf, smods)
  self:read(buf)
  if view.global(spec, self.app.options.views) then
    list.path = "all"
    return
  end
  self.project.path(cwd, function(path)
    list.path = path
    if self.shown[buf] == list and list.arranged then
      self:draw(buf)
    end
  end)
end

return M
