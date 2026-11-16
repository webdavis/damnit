local rows = require("damnit.list.rows")
local tree = require("damnit.list.tree")
local view = require("damnit.list.view")

local Lists = {}
Lists.__index = Lists

local M = {}

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
function M.new(app, open_task)
  local lists = setmetatable({ app = app, open_task = open_task, shown = {} }, Lists)
  lists.group = vim.api.nvim_create_augroup("damnit.list", { clear = true })
  vim.api.nvim_create_autocmd("User", {
    group = lists.group,
    pattern = "DamChanged",
    callback = function()
      lists:changed()
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
    local arranged = tree.arranged(answer.objects)
    self.app.screen:draw(buf, rows.render(arranged))
    list.oids = vim.tbl_map(function(row)
      return row.object.oid
    end, arranged)
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
  self.shown[buf] = { call = { args = args, stdin = stdin, cwd = cwd, key = title }, oids = {} }
  self.app.screen:show(buf, smods)
  self:read(buf)
end

return M
