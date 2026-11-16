local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

return {
  [":Dam status opens dam://status with dam's sections"] = function()
    local _, fake = status.opened()
    assert(vim.api.nvim_buf_get_name(0) == "dam://status", vim.api.nvim_buf_get_name(0))
    assert(vim.bo.filetype == "damstatus", vim.bo.filetype)
    assert(vim.deep_equal(fake.of("status")[1].argv, { "dam", "status", "-o", "json" }))
    assert(buffer.lines()[1] == "Conflicts (1)", vim.inspect(buffer.lines()))
  end,

  [":Dam alone opens the status window"] = function()
    local _, fake = status.defined()
    status.open(fake, "Dam")
    assert(vim.api.nvim_buf_get_name(0) == "dam://status", vim.api.nvim_buf_get_name(0))
  end,

  ["the status window runs dam in the folder it was opened in"] = function()
    local _, fake = status.defined()
    local opened_in = status.folder()
    status.open(fake)
    status.folder()
    status.on("  update call mom back  [subject]")
    buffer.feed("s")
    status.sent(fake, "add")
    buffer.feed("r")
    buffer.wait_for(function()
      return #fake.of("status") >= 2
    end)
    for _, call in ipairs(vim.list_extend(fake.of("status"), fake.of("add"))) do
      assert(call.cwd == opened_in, call.cwd)
    end
  end,

  ["DamChanged rereads it"] = function()
    local app, fake = status.opened()
    fake.reply("status", fixtures.answer("status-empty.json"))
    app.say:changed("/another/folder")
    buffer.wait_for(function()
      return buffer.lines()[1] == "nothing changed, staged, waiting or unpushed"
    end)
  end,

  ["a change sent from the window reads it again"] = function()
    local _, fake = status.opened()
    fake.reply("status", fixtures.answer("status-empty.json"))
    status.on("  update call mom back  [subject]")
    buffer.feed("s")
    buffer.wait_for(function()
      return #buffer.lines() == 1
    end)
  end,

  ["a wiped status window is not read again"] = function()
    local app, fake = status.opened()
    vim.cmd.bwipeout({ bang = true })
    app.say:changed("/")
    vim.wait(30)
    assert(#fake.of("status") == 1, vim.inspect(fake.of("status")))
  end,
}
