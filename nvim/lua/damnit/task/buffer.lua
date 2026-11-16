local edit_flags = require("damnit.task.edit_flags")
local errors = require("damnit.kernel.error")
local header = require("damnit.task.header")

local NAME = "dam://task/"

local Tasks = {}
Tasks.__index = Tasks

local M = {}

---@param app damnit.App
function M.new(app)
  local tasks = setmetatable({ app = app, shown = {}, namespace = vim.api.nvim_create_namespace("damnit.task") }, Tasks)
  local group = vim.api.nvim_create_augroup("damnit.task", { clear = true })
  local events = {
    BufReadCmd = Tasks.read,
    BufWriteCmd = Tasks.write,
    BufWipeout = function(_, buf)
      tasks.shown[buf] = nil
    end,
  }
  for event, run in pairs(events) do
    vim.api.nvim_create_autocmd(event, {
      group = group,
      pattern = NAME .. "*",
      callback = function(args)
        run(tasks, args.buf)
      end,
    })
  end
  return tasks
end

function Tasks:task_of(buf)
  self.shown[buf] = self.shown[buf] or { oid = vim.api.nvim_buf_get_name(buf):sub(#NAME + 1), cwd = vim.fn.getcwd() }
  return self.shown[buf]
end

function Tasks:draw_virtual(buf, virt)
  vim.api.nvim_buf_clear_namespace(buf, self.namespace, 0, -1)
  local virt_lines = vim.tbl_map(function(text)
    return { { text, "Comment" } }
  end, virt)
  vim.api.nvim_buf_set_extmark(buf, self.namespace, 0, 0, { virt_lines = virt_lines, virt_lines_above = true })
  for _, win in ipairs(vim.fn.win_findbuf(buf)) do
    vim.api.nvim_win_call(win, function()
      vim.fn.winrestview({ topline = 1, topfill = #virt })
    end)
  end
end

---@param buf integer
---@param object damnit.Object
function Tasks:draw(buf, object)
  local lines, virt = header.lines(object)
  self:task_of(buf).before = header.parse(lines)
  self.app.screen:draw(buf, lines, {})
  vim.bo[buf].modifiable = true
  self:draw_virtual(buf, virt)
end

---@param buf integer
function Tasks:read(buf)
  local task = self:task_of(buf)
  vim.bo[buf].buftype = "acwrite"
  vim.bo[buf].swapfile = false
  vim.bo[buf].filetype = "damtask"
  self.app.dam:run({ args = { "show", task.oid }, cwd = task.cwd, key = NAME .. task.oid }, function(answer, err)
    if err then
      return self.app.say:error(err)
    end
    if vim.api.nvim_buf_is_valid(buf) then
      self:draw(buf, answer)
    end
  end)
end

function Tasks:answered(buf, after, answer)
  if not vim.api.nvim_buf_is_valid(buf) then
    return
  end
  if answer.hash then
    self:task_of(buf).before = after
    vim.bo[buf].modified = false
    return self.app.say:info(("sent %q, waiting for the owner to take it"):format(answer.subject))
  end
  self:draw(buf, answer)
end

---@param buf integer
function Tasks:write(buf)
  local task = self:task_of(buf)
  local after, problem = header.parse(vim.api.nvim_buf_get_lines(buf, 0, -1, false))
  if not after then
    return self.app.say:error(errors.plugin(problem))
  end
  if not task.before then
    return self.app.say:warn("the task is still being read; write again once it shows")
  end
  local flags = edit_flags.diff(task.before, after)
  if #flags == 0 then
    vim.bo[buf].modified = false
    return
  end
  local call = { args = vim.list_extend({ "edit", task.oid }, flags), cwd = task.cwd }
  self.app.dam:write(call, function(answer, err)
    if not err then
      self:answered(buf, after, answer)
    end
  end)
end

---@param oid string
---@param cwd string
---@param smods? table
function Tasks:open(oid, cwd, smods)
  local buf = vim.fn.bufadd(NAME .. oid)
  if not vim.api.nvim_buf_is_loaded(buf) then
    self.shown[buf] = { oid = oid, cwd = cwd }
  end
  self.app.screen:show(buf, smods)
end

return M
