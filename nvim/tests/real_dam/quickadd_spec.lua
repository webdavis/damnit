local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")
local real_dam = require("helpers.real_dam")
local wait_for = buffer.wait_for

local function quick_added(sandbox)
  local app = real_dam.app()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  local read = helper.call(app, { args = { "new", "buy oat milk", "--due", "fri", "--preview" }, cwd = sandbox })
  wait_for(function()
    return read.done
  end, 20000)
  local due = read.answer.changes[1].due
  vim.cmd.lcd(sandbox)
  vim.cmd("Dam list")
  prompt.type("abuy oat milk --due fri")
  prompt.footer_reads(('task "buy oat milk" · due %s'):format(due), 20000)
  prompt.more("<CR>")
  wait_for(function()
    return vim.tbl_contains(buffer.lines(), "[ ] buy oat milk")
  end, 20000)
  prompt.type("obuy the carton")
  assert(prompt.line():find("^buy the carton %-%-parent %x%x%x%x%x%x%x$"), prompt.line())
  prompt.more("<CR>")
  wait_for(function()
    return vim.tbl_contains(buffer.lines(), "  [ ] buy the carton")
  end, 20000)
end

return {
  ["quick add with --due fri shows dam's date and the task appears; o makes a child under its short oid"] = function()
    return real_dam.with_real_dam(quick_added)
  end,
}
