local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local list = require("helpers.list")

return {
  ["a change dam answers after its list was wiped ends without an error"] = function()
    local _, fake = list.opened()
    fake.reply("done", vim.tbl_extend("force", fixtures.answer("done.json"), { hold = true }))
    list.on(buffer.lines()[1])
    buffer.feed("x")
    buffer.wait_for(function()
      return #fake.of("done") == 1
    end)
    vim.cmd("bwipeout!")
    vim.v.errmsg = ""
    local errors = {}
    local schedule = vim.schedule
    vim.schedule = function(run)
      schedule(function()
        local ok, err = pcall(run)
        if not ok then
          table.insert(errors, err)
        end
      end)
    end
    fake.release(fake.of("done")[1])
    vim.wait(100)
    vim.schedule = schedule
    assert(#errors == 0, vim.inspect(errors))
  end,
}
