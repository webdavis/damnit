local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local M = {}

M.WINDOWS = { week = "agenda-next-week.json", today = "agenda-day.json" }

local function answering(argv)
  local from = argv[vim.fn.index(argv, "--from") + 2]
  return fixtures.answer(M.WINDOWS[from] or "agenda-week.json")
end

---@param command? string
---@return damnit.App app, table fake, string[] opened the oids <CR> opened
function M.opened(command)
  local app, fake = helper.new()
  local opened = {}
  local list = require("damnit.list").new(app, function(oid)
    table.insert(opened, oid)
  end)
  require("damnit.command").define(app, { list, require("damnit.agenda").new(app, list.show) })
  fake.reply("agenda", answering)
  fake.reply("list", fixtures.answer("list-overdue.json"))
  fake.reply("done", fixtures.answer("done.json"))
  fake.reply("filter", fixtures.answer("filter-list.json"))
  for _, verb in ipairs({ "category", "remote", "label", "path" }) do
    fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", 2))
  end
  vim.cmd(command or "Dam agenda")
  buffer.wait_for(function()
    return #fake.of("agenda") > 0 and #buffer.lines() > 1
  end)
  return app, fake, opened
end

---@param fake table
---@return string[]
function M.last_window(fake)
  local calls = fake.of("agenda")
  local argv = calls[#calls].argv
  return { argv[vim.fn.index(argv, "--from") + 2], argv[vim.fn.index(argv, "--to") + 2] }
end

---@param text string
function M.on(text)
  local row = vim.fn.index(buffer.lines(), text) + 1
  assert(row > 0, ("no line %q in %s"):format(text, vim.inspect(buffer.lines())))
  vim.api.nvim_win_set_cursor(0, { row, 0 })
end

return M
