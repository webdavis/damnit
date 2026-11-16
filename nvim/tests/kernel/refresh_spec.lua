local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local function opened_list(now)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  require("damnit.kernel.refresh").new(app.say, now)
  fake.reply("list", fixtures.answer("list.json"))
  vim.cmd("Dam list")
  buffer.wait_for(function()
    return #fake.of("list") == 1 and #buffer.lines() > 1
  end)
  return fake
end

local function settled_reads(fake)
  vim.wait(50)
  vim.api.nvim_del_augroup_by_name("damnit.refresh")
  return #fake.of("list")
end

return {
  ["focus regained makes an open list read again, and twice within a second makes it read once"] = function()
    local fake = opened_list()
    vim.api.nvim_exec_autocmds("FocusGained", {})
    vim.api.nvim_exec_autocmds("FocusGained", {})
    assert(settled_reads(fake) == 2, vim.inspect(fake.of("list")))
  end,

  ["focus regained a second after the last reads again"] = function()
    local clock = 0
    local fake = opened_list(function()
      return clock
    end)
    vim.api.nvim_exec_autocmds("FocusGained", {})
    clock = 1000
    vim.api.nvim_exec_autocmds("FocusGained", {})
    assert(settled_reads(fake) == 3, vim.inspect(fake.of("list")))
  end,
}
