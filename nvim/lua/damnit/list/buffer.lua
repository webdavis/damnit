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
  require("damnit.list.edits"),
  require("damnit.list.filterbar")
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
  local rows, err = self:rows_of(list, answers)
  if err then
    return self.app.say:error(err)
  end
  list.arranged, list.answers = rows, answers
  list.oids = vim.tbl_map(function(row)
    return row.object and not row.readonly and row.object.oid or false
  end, rows)
  self:draw(buf)
  vim.cmd.redraw()
  self:refit(buf)
  self:drawn(list)
end

---@param list table
---@param hook "segments"|"bar"
---@return function|nil the source's hook, while the list reads the source's own calls
function Lists.sourced(list, hook)
  return list.parts == list.own_parts and list.source[hook] or nil
end

---@param list table
---@param answers { answer?: table, err?: damnit.Error }[]
---@return damnit.Row[]|nil rows, damnit.Error|nil err
function Lists:rows_of(list, answers)
  if not self.sourced(list, "segments") then
    return sections.arranged(list.parts, answers, list.grouped_by)
  end
  local segments, err = list.source.segments(answers)
  return segments and sections.rows(segments), err
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
---@param folder? string the buffer name before the view's name, such as dam://sidebar/
function Lists:open(name, cwd, smods, built_in, folder)
  local spec = built_in or name or self.app.options.default_view
  name = name or (type(spec) == "string" and spec) or "default"
  local title = (folder or (built_in and "dam://" or "dam://list/")) .. name
  self:show({ name = name, title = title, view = spec }, cwd, smods)
end

---@class damnit.ListSource
---@field name string
---@field title? string the buffer's name; dam://<name> when left out
---@field filetype? string damlist when left out
---@field view? string|damnit.ViewSpec|damnit.ViewSpec[] read as :Dam list reads a view
---@field parts? damnit.ListPart[] the calls to make in place of a view's
---@field segments? fun(answers: { answer?: table, err?: damnit.Error }[]): damnit.Segment[]|nil, damnit.Error|nil
---@field bar? fun(answers: { answer?: table, err?: damnit.Error }[], marked: integer): string[][]
---@field keys? fun(window: damnit.ListWindow): damnit.KeySpec[]

---@param source damnit.ListSource
---@param cwd string
---@param smods? table
function Lists:show(source, cwd, smods)
  local title = source.title or ("dam://" .. source.name)
  local buf = self.app.screen:scratch(title, source.filetype or "damlist")
  local list = { name = source.name, source = source, call = { cwd = cwd, key = title }, oids = {}, waiting = {} }
  list.view = source.view
  if source.parts then
    list.parts = source.parts
  else
    list.parts, list.grouped_by = view.parts(source.view, self.app.options.views)
  end
  list.own_parts = list.parts
  local fresh = not self.shown[buf]
  self.shown[buf] = list
  if fresh then
    self:bind(buf)
  end
  self.app.screen:show(buf, smods)
  self:read(buf)
  if view.global(source.view, self.app.options.views) then
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
