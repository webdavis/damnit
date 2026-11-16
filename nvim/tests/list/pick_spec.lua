local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local MILK = ("1"):rep(40)

local function defined(options)
  local app, fake = helper.new(options)
  local answers = { list = "list.json", filter = "filter-list.json", status = "status.json", log = "log.json" }
  for verb, name in pairs(answers) do
    fake.reply(verb, fixtures.answer(name))
  end
  for _, verb in ipairs({ "category", "remote", "label", "path" }) do
    fake.reply(verb, { code = 0, stdout = "{}", stderr = "" })
  end
  local opened = {}
  local list = require("damnit.list").new(app, function(oid, cwd)
    table.insert(opened, { oid = oid, cwd = cwd })
  end)
  local log = require("damnit.log").new(app, function() end)
  require("damnit.command").define(app, { list, require("damnit.status").new(app, function() end, log.open), log })
  return fake, opened
end

local function choosing(pick, run)
  local select = vim.ui.select
  local offered = {}
  vim.ui.select = function(items, opts, on_choice)
    offered.texts = vim.tbl_map(opts.format_item or tostring, items)
    offered.prompt = opts.prompt
    local index = vim.fn.index(offered.texts, pick) + 1
    on_choice(items[index] or nil)
  end
  local ok, err = pcall(run)
  vim.ui.select = select
  assert(ok, err)
  return offered
end

local function list_read(fake, count)
  buffer.wait_for(function()
    return #fake.of("list") >= count and vim.bo.filetype == "damlist" and #buffer.lines() > 1
  end)
end

local VIEWS = { "today · built in", "all · view in your options", "today · saved filter" }

return {
  [":Dam pick offers today, the views in the options and dam's saved filters"] = function()
    defined({ views = { all = { done = "any" } } })
    local offered = choosing(nil, function()
      vim.cmd("Dam pick")
    end)
    assert(vim.deep_equal(offered.texts, VIEWS), vim.inspect(offered.texts))
  end,

  ["choosing a saved filter opens it as a list"] = function()
    local fake = defined()
    choosing("today · saved filter", function()
      vim.cmd("Dam pick")
    end)
    list_read(fake, 1)
    assert(vim.api.nvim_buf_get_name(0) == "dam://list/today", vim.api.nvim_buf_get_name(0))
    assert(vim.deep_equal(fake.of("list")[1].argv, { "dam", "list", "--filter", "today", "-o", "json" }))
  end,

  ["choosing the built-in today opens its sections"] = function()
    local fake = defined()
    choosing("today · built in", function()
      vim.cmd("Dam pick")
    end)
    list_read(fake, 4)
    assert(vim.api.nvim_buf_get_name(0) == "dam://today", vim.api.nvim_buf_get_name(0))
  end,

  [":Dam pick <view> offers the view's rows and opens the chosen task"] = function()
    local fake, opened = defined({ views = { all = { done = "any" } } })
    local offered = choosing("buy oat milk  2026-09-25  home/", function()
      vim.cmd("Dam pick all")
      buffer.wait_for(function()
        return #opened > 0
      end)
    end)
    assert(vim.deep_equal(vim.json.decode(fake.of("list")[1].stdin), { done = "any" }), fake.of("list")[1].stdin)
    local rows = { "call mom", "buy oat milk  2026-09-25  home/", "pack the bag  home/buy oat milk/" }
    assert(vim.deep_equal(offered.texts, rows), vim.inspect(offered.texts))
    assert(opened[1].oid == MILK, vim.inspect(opened))
  end,

  ["go in a list window offers every view"] = function()
    local fake = defined()
    vim.cmd("Dam list")
    list_read(fake, 1)
    local offered = choosing(nil, function()
      buffer.feed("go")
    end)
    assert(vim.deep_equal(offered.texts, { VIEWS[1], VIEWS[3] }), vim.inspect(offered.texts))
  end,

  ["go in the status window offers every view"] = function()
    local fake = defined()
    vim.cmd("Dam status")
    buffer.wait_for(function()
      return #fake.of("status") > 0 and #buffer.lines() > 1
    end)
    local offered = choosing(nil, function()
      buffer.feed("go")
    end)
    assert(vim.deep_equal(offered.texts, { VIEWS[1], VIEWS[3] }), vim.inspect(offered.texts))
  end,

  ["go in the log window offers every view"] = function()
    local fake = defined()
    vim.cmd("Dam log")
    buffer.wait_for(function()
      return #fake.of("log") > 0 and vim.bo.filetype == "damlog" and #buffer.lines() > 1
    end)
    local offered = choosing(nil, function()
      buffer.feed("go")
    end)
    assert(vim.deep_equal(offered.texts, { VIEWS[1], VIEWS[3] }), vim.inspect(offered.texts))
  end,
}
