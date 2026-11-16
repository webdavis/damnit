local rows = require("damnit.log.rows")

local NAME = "dam://log"

local Log = {}
Log.__index = Log

local M = {}

---@param app damnit.App
---@param open_status fun(cwd: string)
function M.new(app, open_status)
  local log = setmetatable({ app = app, open_status = open_status, commits = {}, shown = {} }, Log)
  log.group = vim.api.nvim_create_augroup("damnit.log", { clear = true })
  vim.api.nvim_create_autocmd("User", {
    group = log.group,
    pattern = "DamChanged",
    callback = function()
      if log.buf then
        log:read()
      end
    end,
  })
  vim.api.nvim_create_autocmd("WinResized", {
    group = log.group,
    callback = function()
      if log.buf and log.app.screen:text_width(log.buf) ~= log.width then
        log:draw()
      end
    end,
  })
  return log
end

---@param at? string the commit to put the cursor on
function Log:read(at)
  local buf = self.buf
  self.app.dam:run({ args = { "log" }, cwd = self.cwd, key = NAME }, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    if buf ~= self.buf then
      return
    end
    self.commits = answer.commits
    self:draw()
    for n, commit in ipairs(self.commits) do
      if commit.id == at and vim.api.nvim_get_current_buf() == buf then
        vim.api.nvim_win_set_cursor(0, { n, 0 })
      end
    end
  end)
end

function Log:draw()
  local count = #self.commits
  local winbar =
    { { " log", "DamHeader" }, { "", cut = true }, { " · " }, { count .. " commit" .. (count == 1 and "" or "s") } }
  self.app.screen:window(self.buf, { winbar = self.app.screen.bar(winbar), wrap = false })
  self.width = self.app.screen:text_width(self.buf)
  local lines, marks = rows.render(self.commits, self.width)
  for n, commit in ipairs(self.commits) do
    if self.shown[commit.id] then
      table.insert(marks, { line = n - 1, virt_lines = self.shown[commit.id] })
    end
  end
  self.app.screen:draw(self.buf, count > 0 and lines or { "no commits yet" }, marks)
end

---@return damnit.Commit|nil commit, integer line
function Log:commit()
  local line = vim.api.nvim_win_get_cursor(0)[1]
  return self.commits[line], line
end

function Log:toggle()
  local commit = self:commit()
  if not commit then
    return self.app.say:warn("<CR> shows the changes of the commit under the cursor")
  end
  if self.shown[commit.id] then
    self.shown[commit.id] = nil
    return self:draw()
  end
  self.app.dam:run({ args = { "show", commit.id }, cwd = self.cwd }, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    self.shown[commit.id] = rows.changes(answer)
    self:draw()
  end)
end

---@param buf integer
function Log:bind(buf)
  self.app.keys:bind(buf, {
    {
      lhs = "<CR>",
      desc = "show or hide the commit's changes",
      group = "log",
      run = function()
        self:toggle()
      end,
    },
  })
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.buf, self.commits, self.shown = nil, {}, {}
    end,
  })
end

---@param cwd string
---@param smods? table
---@param at? string the commit to put the cursor on
function Log:open(cwd, smods, at)
  local buf = self.app.screen:scratch(NAME, "damlog")
  if buf ~= self.buf then
    self:bind(buf)
  end
  self.buf, self.cwd = buf, cwd
  self.app.screen:show(buf, smods)
  self:read(at)
end

return M
