local project = require("damnit.list.project")
local sections = require("damnit.list.sections")
local view = require("damnit.list.view")

local Lists = vim.tbl_extend(
  "error",
  {},
  require("damnit.list.drawing"),
  require("damnit.list.bindings"),
  require("damnit.list.landing"),
  require("damnit.list.change_keys"),
  require("damnit.list.edits")
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
---@param answers { answer?: damnit.ListAnswer, err?: damnit.Error }[]
function Lists:arrange(buf, answers)
  local list = self.shown[buf]
  local rows, err = sections.arranged(list.parts, answers, list.grouped_by)
  if err then
    return self.app.say:error(err)
  end
  list.arranged = rows
  list.oids = vim.tbl_map(function(row)
    return row.object and row.object.oid or false
  end, rows)
  self:draw(buf)
  vim.cmd.redraw()
  self:refit(buf)
  self:drawn(list)
end

---@param buf integer
function Lists:read(buf)
  local list = self.shown[buf]
  local answers, reading = {}, {}
  list.reading = reading
  for n, part in ipairs(list.parts) do
    local lane = #list.parts > 1 and (list.call.key .. "#" .. n) or list.call.key
    local call = { args = part.args, stdin = part.stdin, cwd = list.call.cwd, key = lane }
    self.app.dam:run(call, function(answer, err)
      if list.reading ~= reading or not vim.api.nvim_buf_is_valid(buf) then
        return
      end
      answers[n] = { answer = answer, err = err }
      if vim.tbl_count(answers) == #list.parts then
        self:arrange(buf, answers)
      end
    end)
  end
end

function Lists:changed()
  for buf in pairs(self.shown) do
    self:read(buf)
  end
end

---@param name string|nil
---@param cwd string
---@param smods? table
---@param built_in? damnit.ViewSpec[] a view the plugin names, shown as dam://<name>
function Lists:open(name, cwd, smods, built_in)
  local spec = built_in or name or self.app.options.default_view
  name = name or (type(spec) == "string" and spec) or "default"
  local title = (built_in and "dam://" or "dam://list/") .. name
  local buf = self.app.screen:scratch(title, "damlist")
  if not self.shown[buf] then
    self:bind(buf)
  end
  local list = { name = name, call = { cwd = cwd, key = title }, oids = {}, view = spec, waiting = {} }
  list.parts, list.grouped_by = view.parts(spec, self.app.options.views)
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
