local folds = require("damnit.kernel.folds")
local sections = require("damnit.status.sections")
local winbar = require("damnit.status.winbar")

local NAME = "dam://status"

local Status = vim.tbl_extend(
  "error",
  {},
  require("damnit.status.keys"),
  require("damnit.status.confirms"),
  require("damnit.status.refusals"),
  require("damnit.status.details")
)
Status.__index = Status

local M = {}

---@param app damnit.App
---@param writes damnit.StatusWrites
---@param open_task fun(oid: string, cwd: string)
function M.new(app, writes, open_task)
  local status = setmetatable({ app = app, writes = writes, open_task = open_task, targets = {}, shown = {} }, Status)
  status.group = vim.api.nvim_create_augroup("damnit.status", { clear = true })
  vim.api.nvim_create_autocmd("User", {
    group = status.group,
    pattern = "DamChanged",
    callback = function()
      if status.buf then
        status:read()
      end
    end,
  })
  vim.api.nvim_create_autocmd("WinResized", {
    group = status.group,
    callback = function()
      status:refit()
    end,
  })
  return status
end

function Status:read()
  local buf = self.buf
  self.app.dam:run({ args = { "status", "--full" }, cwd = self.cwd, key = NAME }, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    if buf ~= self.buf then
      return
    end
    self.answer = answer
    self:draw()
  end)
end

---@param count integer
---@param targets table<integer, damnit.StatusTarget>
---@return damnit.FoldLine[]
local function fold_lines(count, targets)
  local lines = {}
  for n = 1, count do
    local section = targets[n] and targets[n].kind == "section" and targets[n].section
    lines[n] = targets[n] and { header = section and true, depth = 0, key = section or nil } or {}
  end
  return lines
end

function Status:draw()
  local buf = self.buf
  local window = { winbar = self.app.screen.bar(winbar.chunks(self.answer)), wrap = false }
  self.width = self.app.screen:text_width(buf)
  local lines, marks, targets = sections.render(self.answer, self.width)
  vim.list_extend(marks, self:detail_marks(targets))
  folds.redraw(buf, fold_lines(#lines, targets), function()
    self.app.screen:window(buf, vim.tbl_extend("error", window, folds.window))
    self.app.screen:draw(buf, lines, marks)
  end)
  self.targets = targets
end

function Status:refit()
  if self.buf and self.answer and self.app.screen:text_width(self.buf) ~= self.width then
    self:draw()
  end
end

---@return damnit.StatusTarget|nil
function Status:target()
  return self.targets[vim.api.nvim_win_get_cursor(0)[1]]
end

function Status:open_target_task()
  local target = self:target()
  if not (target and target.oid) then
    return self.app.say:warn("<CR> opens the task of a change, waiting item or conflict row")
  end
  self.open_task(target.oid, self.cwd)
end

---@param cwd string
---@param smods? table
function Status:open(cwd, smods)
  local buf = self.app.screen:scratch(NAME, "damstatus")
  if buf ~= self.buf then
    self:bind(buf)
  end
  self.buf, self.cwd = buf, cwd
  self.app.screen:show(buf, smods)
  self:read()
end

return M
