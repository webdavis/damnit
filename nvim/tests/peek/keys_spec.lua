local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local status = require("helpers.status")

local MILK = ("1"):rep(40)
local MOM = ("2"):rep(40)

local function shown_oid(fake)
  buffer.wait_for(function()
    return #fake.of("show") > 0
  end)
  return fake.of("show")[1].argv[3]
end

local function app_with_peek(options)
  local app, fake = helper.new(options)
  fake.reply("show", fixtures.answer("show.json"))
  fake.reply("log", fixtures.answer("log.json"))
  return app, fake, require("damnit.peek").new(app, function() end)
end

return {
  ["K on a list row peeks at that row's task"] = function()
    local app, fake = app_with_peek()
    fake.reply("list", fixtures.answer("list.json"))
    require("damnit.command").define(app, { require("damnit.list").new(app, function() end) })
    vim.cmd("Dam list")
    buffer.wait_for(function()
      return #buffer.lines() > 1
    end)
    vim.api.nvim_win_set_cursor(0, { vim.fn.index(buffer.lines(), "[ ] p2 buy oat milk") + 1, 0 })
    buffer.feed("K")
    assert(shown_oid(fake) == MILK, shown_oid(fake))
  end,

  ["K on a status change row peeks at its task"] = function()
    local app, fake = app_with_peek()
    fake.reply("status", fixtures.answer("status.json"))
    require("damnit.command").define(app, { require("damnit.status").new(app, function() end) })
    status.open(fake)
    status.on("  update  call mom back")
    buffer.feed("K")
    assert(shown_oid(fake) == MOM, shown_oid(fake))
  end,

  ["K on a section header warns and reads nothing"] = function()
    local notes = buffer.capture_notes()
    local app, fake = app_with_peek({ views = { all = { { title = "All" }, { title = "Again" } } } })
    fake.reply("list", fixtures.answer("list.json"))
    require("damnit.command").define(app, { require("damnit.list").new(app, function() end) })
    vim.cmd("Dam list all")
    buffer.wait_for(function()
      return #buffer.lines() > 1
    end)
    assert(buffer.lines()[1] == "All (3)", vim.inspect(buffer.lines()))
    vim.api.nvim_win_set_cursor(0, { 1, 0 })
    buffer.feed("K")
    vim.wait(30)
    assert(#fake.of("show") == 0, vim.inspect(fake.of("show")))
    assert(notes[1].text == "K acts on task rows; this line holds no task", vim.inspect(notes))
  end,

  ["K is listed by g? under the window's task keys"] = function()
    local app, fake = app_with_peek()
    fake.reply("status", fixtures.answer("status.json"))
    require("damnit.command").define(app, { require("damnit.status").new(app, function() end) })
    status.open(fake)
    local status_buf = vim.api.nvim_get_current_buf()
    local map = vim.fn.maparg("K", "n", false, true)
    assert(map.buffer == 1 and map.desc == "peek at the task, or enter the peek", vim.inspect(map))
    buffer.feed("g?")
    local lines = buffer.lines()
    local heading = vim.fn.index(lines, "status")
    assert(
      heading >= 0 and vim.fn.index(lines, "  K  peek at the task, or enter the peek") > heading,
      vim.inspect(lines)
    )
    assert(status_buf ~= vim.api.nvim_get_current_buf())
  end,
}
