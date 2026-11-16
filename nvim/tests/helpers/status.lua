local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local M = {}

local ANSWERS = {
  status = "status.json",
  add = "status-add.json",
  reset = "status-reset.json",
  rm = "status-decline.json",
  commit = "status-commit.json",
  push = "push.json",
  pull = "pull.json",
  log = "log.json",
}

---@param open_task? fun(oid: string, cwd: string)
---@return damnit.App app, table fake
function M.defined(open_task)
  local app, fake = helper.new()
  local status = require("damnit.status").new(app, open_task or function() end)
  require("damnit.command").define(app, { status })
  for verb, name in pairs(ANSWERS) do
    fake.reply(verb, fixtures.answer(name))
  end
  fake.reply("push", function(argv)
    return fixtures.answer(vim.tbl_contains(argv, "--preview") and "push-preview.json" or "push.json")
  end)
  fake.reply("restore", function(argv)
    return fixtures.answer(vim.tbl_contains(argv, "--preview") and "restore-preview.json" or "restore.json")
  end)
  return app, fake
end

---@param fake table
---@param command? string
function M.open(fake, command)
  local before = #fake.of("status")
  vim.cmd(command or "Dam status")
  buffer.wait_for(function()
    return #fake.of("status") > before and #buffer.lines() > 1
  end)
end

---@param open_task? fun(oid: string, cwd: string)
---@return damnit.App app, table fake
function M.opened(open_task)
  local app, fake = M.defined(open_task)
  M.open(fake)
  return app, fake
end

---@param text string
function M.on(text)
  local row = vim.fn.index(buffer.lines(), text) + 1
  assert(row > 0, ("no row %q in %s"):format(text, vim.inspect(buffer.lines())))
  vim.api.nvim_win_set_cursor(0, { row, 0 })
end

---@param fake table
---@param verb string
---@return string[] argv between dam and -o json
function M.sent(fake, verb)
  buffer.wait_for(function()
    return #fake.of(verb) > 0
  end)
  local argv = fake.of(verb)[#fake.of(verb)].argv
  return vim.list_slice(argv, 2, #argv - 2)
end

---@param fake table
---@param verb string
function M.nothing_sent(fake, verb)
  vim.wait(30)
  assert(#fake.of(verb) == 0, vim.inspect(fake.of(verb)))
end

---@return string
function M.folder()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  return vim.fn.getcwd()
end

return M
