local look = require("damnit.list.look")
local project = require("damnit.list.project")
local tree = require("damnit.list.tree")
local view = require("damnit.list.view")
local winbar = require("damnit.list.winbar")

local Lists = {}
Lists.__index = Lists

local M = {}

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
function M.new(app, open_task)
  local lists = setmetatable({ app = app, open_task = open_task, shown = {}, project = project.new(app) }, Lists)
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
function Lists:draw(buf)
  local list = self.shown[buf]
  list.width = self.app.screen:text_width(buf)
  self.app.screen:draw(buf, look.render(list.arranged, self.app.options.signs, list.width))
  local objects = vim.tbl_map(function(row)
    return row.object
  end, list.arranged)
  local bar = winbar.chunks(list.name, list.path or vim.fs.basename(list.call.cwd), objects)
  self.app.screen:window(buf, { winbar = self.app.screen.bar(bar), signcolumn = "auto:2", wrap = false })
end

---@param buf integer
function Lists:refit(buf)
  local list = self.shown[buf]
  if list and list.arranged and vim.api.nvim_buf_is_valid(buf) and self.app.screen:text_width(buf) ~= list.width then
    self:draw(buf)
  end
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
  end)
end

function Lists:changed()
  for buf in pairs(self.shown) do
    self:read(buf)
  end
end

---@param buf integer
function Lists:bind(buf)
  self.app.keys:bind(buf, {
    {
      lhs = "<CR>",
      desc = "open the task",
      run = function()
        local list = self.shown[buf]
        local oid = list.oids[vim.api.nvim_win_get_cursor(0)[1]]
        if oid then
          self.open_task(oid, list.call.cwd)
        end
      end,
    },
    {
      lhs = "r",
      desc = "read the list again",
      run = function()
        self:read(buf)
      end,
    },
  })
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.shown[buf] = nil
    end,
  })
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
  list.oids = {}
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
