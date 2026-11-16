local buffer = require("helpers.buffer")
local real_dam = require("helpers.real_dam")

local EXPECTED = {
  "Overdue (1)",
  "[ ] renew the passport",
  "Today (1)",
  "[ ] call the bank",
  "Next 7 days (2)",
  "[ ] send the invoice",
  "[ ] book the van",
  "No date (3)",
  "[ ] plan the move",
  "  [ ] pack the books",
  "[ ] read the lease",
}

local function a_week_of_tasks(app, sandbox)
  local made = real_dam.made
  made(app, { "new", "renew the passport", "--due", "today-2" }, sandbox)
  made(app, { "new", "call the bank", "--due", "today" }, sandbox)
  made(app, { "new", "send the invoice", "--due", "today+3" }, sandbox)
  made(app, { "new", "read the lease" }, sandbox)
  local parent = made(app, { "new", "plan the move" }, sandbox)
  made(app, { "new", "pack the books", "--parent", parent }, sandbox)
  made(app, { "new", "book the van", "--parent", parent, "--due", "today+3" }, sandbox)
end

return {
  [":Dam today puts each task under its header, a child due elsewhere a root there"] = function()
    return real_dam.with_real_dam(function(sandbox)
      local app = real_dam.app()
      require("damnit.command").define(app, { require("damnit.list").new(app) })
      a_week_of_tasks(app, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam today")
      buffer.wait_for(function()
        return #buffer.lines() == #EXPECTED
      end, 20000)
      assert(vim.deep_equal(buffer.lines(), EXPECTED), vim.inspect(buffer.lines()))
    end)
  end,
}
