local buffer = require("helpers.buffer")
local helper = require("helpers.app")

local function define()
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.messages").new(app) })
  return app, fake
end

return {
  [":Dam with an unknown subcommand lists the valid ones"] = function()
    define()
    local notes = buffer.capture_notes()
    vim.cmd("Dam lsit")
    assert(#notes == 1 and notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
    assert(notes[1].text:find("lsit", 1, true) and notes[1].text:find("cancel, messages", 1, true), notes[1].text)
  end,

  [":Dam alone says which subcommands exist"] = function()
    define()
    local notes = buffer.capture_notes()
    vim.cmd("Dam")
    assert(#notes == 1 and notes[1].text:find("cancel, messages", 1, true), vim.inspect(notes))
  end,

  [":Dam completes subcommand names"] = function()
    define()
    assert(vim.deep_equal(vim.fn.getcompletion("Dam me", "cmdline"), { "messages" }))
  end,

  [":Dam cancel stops every running call"] = function()
    local app, fake = define()
    fake.reply("pull", { code = 0, stdout = "{}", stderr = "", hold = true })
    local got = helper.call(app, { args = { "pull" }, cwd = "/" })
    buffer.wait_for(function()
      return #fake.of("pull") == 1
    end)
    local notes = buffer.capture_notes()
    vim.cmd("Dam cancel")
    vim.wait(30)
    assert(fake.of("pull")[1].killed and not got.done)
    assert(notes[1].text == "cancelled 1 dam call", vim.inspect(notes))
  end,

  ["the first :Dam sets the plugin up with its defaults"] = function()
    pcall(vim.api.nvim_del_user_command, "Dam")
    package.loaded["damnit"] = nil
    dofile(vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "../../plugin/damnit.lua"))
    vim.cmd("Dam messages")
    assert(vim.deep_equal(buffer.lines(), { "no dam errors yet" }), vim.inspect(buffer.lines()))
  end,
}
