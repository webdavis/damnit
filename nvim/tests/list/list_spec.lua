local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local EXIT_USAGE = 2

local function opened(options, reply)
  local app, fake = helper.new(options)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", fixtures.answer(reply or "list.json"))
  return app, fake
end

local function folder()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  return vim.fn.getcwd()
end

local function wait_for_reads(fake, count)
  buffer.wait_for(function()
    return #fake.of("list") >= count and #buffer.lines() > 1
  end)
end

local function last_list(fake)
  local calls = fake.of("list")
  return calls[#calls]
end

return {
  [":Dam list opens dam://list with dam's rows"] = function()
    local _, fake = opened()
    vim.cmd("Dam list")
    wait_for_reads(fake, 1)
    assert(vim.api.nvim_buf_get_name(0) == "dam://list/default", vim.api.nvim_buf_get_name(0))
    assert(vim.bo.filetype == "damlist", vim.bo.filetype)
    assert(vim.deep_equal(last_list(fake).argv, { "dam", "list", "-o", "json" }), vim.inspect(last_list(fake).argv))
    assert(buffer.lines()[3] == "  [ ] pack the bag", vim.inspect(buffer.lines()))
  end,

  [":Dam list <view> sends the view's table to dam on stdin"] = function()
    local _, fake = opened({ views = { all = { done = "any" } } })
    vim.cmd("Dam list all")
    wait_for_reads(fake, 1)
    local call = last_list(fake)
    assert(vim.api.nvim_buf_get_name(0) == "dam://list/all", vim.api.nvim_buf_get_name(0))
    assert(vim.deep_equal(call.argv, { "dam", "list", "--request", "-", "-o", "json" }), vim.inspect(call.argv))
    assert(vim.deep_equal(vim.json.decode(call.stdin), { done = "any" }), call.stdin)
  end,

  ["the list runs dam in the folder it was opened in"] = function()
    local _, fake = opened()
    local opened_in = folder()
    vim.cmd("Dam list")
    wait_for_reads(fake, 1)
    folder()
    buffer.feed("r")
    wait_for_reads(fake, 2)
    assert(fake.of("list")[1].cwd == opened_in, fake.of("list")[1].cwd)
    assert(last_list(fake).cwd == opened_in, last_list(fake).cwd)
  end,

  ["a failed read shows dam's message and keeps the rows"] = function()
    local _, fake = opened()
    vim.cmd("Dam list")
    wait_for_reads(fake, 1)
    local before = buffer.lines()
    local notes = buffer.capture_notes()
    fake.reply("list", fixtures.refusal("error-usage-selection.json", EXIT_USAGE))
    buffer.feed("r")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text:find("--priority cannot be", 1, true), vim.inspect(notes))
    assert(vim.deep_equal(buffer.lines(), before), vim.inspect(buffer.lines()))
  end,

  ["r reads again"] = function()
    local _, fake = opened()
    vim.cmd("Dam list")
    wait_for_reads(fake, 1)
    fake.reply("list", fixtures.answer("list-tree.json"))
    buffer.feed("r")
    buffer.wait_for(function()
      return buffer.lines()[3] == "[x] water the plants"
    end)
  end,

  ["<CR> names the :Dam task that opens the row"] = function()
    local _, fake = opened()
    vim.cmd("Dam list")
    wait_for_reads(fake, 1)
    local notes = buffer.capture_notes()
    vim.api.nvim_win_set_cursor(0, { 2, 0 })
    buffer.feed("<CR>")
    assert(#notes == 1, vim.inspect(notes))
    assert(notes[1].text:find(":Dam task 1111111111111111111111111111111111111111", 1, true), notes[1].text)
  end,

  ["list completion offers option views and saved filters"] = function()
    local _, fake = opened({ views = { mine = { labels = "me" } } })
    fake.reply("filter", fixtures.answer("filter-list.json"))
    local offered = vim.fn.getcompletion("Dam list ", "cmdline")
    assert(vim.deep_equal(offered, { "mine", "today" }), vim.inspect(offered))
    assert(vim.deep_equal(fake.of("filter")[1].argv, { "dam", "filter", "list", "-o", "json" }))
  end,
}
