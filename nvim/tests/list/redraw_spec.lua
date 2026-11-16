local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local function opened(options)
  local app, fake = helper.new(options)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", fixtures.answer("list.json"))
  return app, fake
end

local function folder()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  return vim.fn.getcwd()
end

local function list(fake, command)
  local before = #fake.of("list")
  vim.cmd(command)
  buffer.wait_for(function()
    return #fake.of("list") > before and #buffer.lines() > 1
  end)
  return vim.api.nvim_get_current_buf()
end

local function reads_after(fake, count)
  buffer.wait_for(function()
    return #fake.of("list") >= count
  end)
  vim.wait(50)
  return vim.list_slice(fake.of("list"), count)
end

return {
  ["DamChanged redraws an open list"] = function()
    local app, fake = opened()
    local cwd = folder()
    list(fake, "Dam list")
    fake.reply("list", fixtures.answer("list-tree.json"))
    app.say:changed(cwd)
    buffer.wait_for(function()
      return #buffer.lines() == 6
    end)
  end,

  ["a change made in another folder redraws an open list"] = function()
    local app, fake = opened()
    local cwd = folder()
    list(fake, "Dam list")
    fake.reply("list", fixtures.answer("list-tree.json"))
    app.say:changed("/another/folder")
    buffer.wait_for(function()
      return #buffer.lines() == 6
    end)
    assert(fake.of("list")[2].cwd == cwd, fake.of("list")[2].cwd)
  end,

  ["a global list redraws on a change from any folder"] = function()
    local app, fake = opened({ views = { everywhere = { global = true } } })
    folder()
    list(fake, "Dam list everywhere")
    fake.reply("list", fixtures.answer("list-tree.json"))
    app.say:changed("/some/project")
    buffer.wait_for(function()
      return #buffer.lines() == 6
    end)
  end,

  ["two open lists both reread, each in its own folder"] = function()
    local app, fake = opened({ views = { all = { done = "any" } } })
    local first = folder()
    list(fake, "Dam list")
    vim.cmd.vsplit()
    local second = folder()
    list(fake, "Dam list all")
    app.say:changed("/another/folder")
    local reads = reads_after(fake, 3)
    local seen = vim.tbl_map(function(call)
      return table.concat(call.argv, " ") .. " in " .. call.cwd
    end, reads)
    table.sort(seen)
    local expected = {
      "dam list --request - -o json in " .. second,
      "dam list -o json in " .. first,
    }
    assert(vim.deep_equal(seen, expected), vim.inspect(seen))
  end,

  ["a list whose buffer was wiped is not read again"] = function()
    local app, fake = opened()
    local cwd = folder()
    local buf = list(fake, "Dam list")
    vim.cmd.bwipeout({ args = { buf }, bang = true })
    app.say:changed(cwd)
    vim.wait(50)
    assert(#fake.of("list") == 1, vim.inspect(fake.of("list")))
  end,
}
