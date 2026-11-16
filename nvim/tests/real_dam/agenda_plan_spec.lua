local buffer = require("helpers.buffer")
local real_dam = require("helpers.real_dam")
local wait_for = buffer.wait_for

local DAY_HEADER = "^%l%l%l %d%d%d%d%-%d%d%-%d%d$"

local function defined()
  local app = real_dam.app()
  local list = require("damnit.list").new(app)
  require("damnit.command").define(app, {
    list,
    require("damnit.agenda").new(app, list.show),
    require("damnit.plan").new(app, list.show),
  })
  return app
end

local function line_after(pattern, row)
  local lines = buffer.lines()
  for n, line in ipairs(lines) do
    if line:find(pattern) and lines[n + 1] == row then
      return n
    end
  end
end

return {
  ["a task due this week appears under its day in :Dam agenda"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app = defined()
      real_dam.made(app, { "new", "call the bank", "--due", "today" }, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam agenda")
      wait_for(function()
        return line_after(DAY_HEADER, "[ ] call the bank") ~= nil
      end, 20000)
    end)
  end,

  ["> in :Dam plan moves a task from Unplanned to today"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app = defined()
      real_dam.made(app, { "new", "read the lease" }, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam plan")
      wait_for(function()
        return line_after("^Unplanned %(1%)$", "[ ] read the lease") ~= nil
      end, 20000)
      vim.api.nvim_win_set_cursor(0, { line_after("^Unplanned", "[ ] read the lease") + 1, 0 })
      buffer.feed(">")
      wait_for(function()
        return line_after("^today · %d%d%d%d%-%d%d%-%d%d %(1%)$", "[ ] read the lease") ~= nil
      end, 20000)
      assert(buffer.lines()[1] == "Unplanned (0)", vim.inspect(buffer.lines()))
    end)
  end,
}
