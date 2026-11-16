local actions = require("damnit.status.actions")
local sections = require("damnit.status.sections")
local winbar = require("damnit.status.winbar")

local NAME = "dam://status"

local Status = {}
Status.__index = Status

local M = {}

---@param app damnit.App
---@param writes damnit.StatusWrites
---@param open_task fun(oid: string, cwd: string)
function M.new(app, writes, open_task)
  local status = setmetatable({ app = app, writes = writes, open_task = open_task, targets = {} }, Status)
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
  self.app.dam:run({ args = { "status" }, cwd = self.cwd, key = NAME }, function(answer, err)
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

function Status:draw()
  local buf = self.buf
  self.app.screen:window(buf, { winbar = self.app.screen.bar(winbar.chunks(self.answer)), wrap = false })
  self.width = self.app.screen:text_width(buf)
  local lines, marks, targets = sections.render(self.answer, self.width)
  self.app.screen:draw(buf, lines, marks)
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

---@param key "s"|"u"|"a"|"x"
function Status:act(key)
  local argv = actions.argv(self:target(), key)
  if not argv then
    return self.app.say:warn(actions.fits(key))
  end
  if key ~= "x" then
    return self.writes.send(argv, self.cwd)
  end
  local row = vim.trim(vim.api.nvim_get_current_line()):gsub("%s+", " ")
  vim.ui.select({ "decline", "keep" }, { prompt = ("decline %s?"):format(row) }, function(choice)
    if choice == "decline" then
      self.writes.send(argv, self.cwd)
    end
  end)
end

function Status:open_target_task()
  local target = self:target()
  if not (target and target.oid) then
    return self.app.say:warn("<CR> opens the task of a change, waiting item or conflict row")
  end
  self.open_task(target.oid, self.cwd)
end

---@param buf integer
function Status:bind(buf)
  local function act(key, desc)
    return {
      lhs = key,
      desc = desc,
      run = function()
        self:act(key)
      end,
    }
  end
  local function call(lhs, desc, run)
    return { lhs = lhs, desc = desc, run = run }
  end
  self.app.keys:bind(buf, {
    act("s", "stage the change, or everything on a header"),
    act("u", "unstage the change, or everything on the Staged header"),
    act("a", "accept the waiting item or set"),
    act("x", "decline the waiting item or set"),
    call("c", "commit the stage", function()
      self.writes.commit(self.cwd)
    end),
    call("P", "push to every remote", function()
      self.writes.push(self.cwd)
    end),
    call("F", "pull from every remote", function()
      self.writes.pull(self.cwd)
    end),
    call("<CR>", "open the task", function()
      self:open_target_task()
    end),
    call("r", "read the status again", function()
      self:read()
    end),
  })
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.buf, self.answer = nil, nil
    end,
  })
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
