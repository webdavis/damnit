local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local EXIT_USAGE, EXIT_REFUSED = 2, 4

local function shown()
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.task").new(app) })
  fake.reply("show", fixtures.answer("show.json"))
  vim.cmd("Dam task abcdef1")
  buffer.wait_for(function()
    return buffer.lines()[1]:find("^subject:") ~= nil
  end)
  return app, fake
end

local function written(fake, reply, line, text)
  fake.reply("edit", reply)
  vim.api.nvim_buf_set_lines(0, line - 1, line, false, { text })
  vim.cmd.write()
  buffer.wait_for(function()
    return #vim.diagnostic.get(0) > 0 or not vim.bo.modified
  end)
end

return {
  ["a refused --due lands as an error on the due: line, once: no notification, one entry in the log"] = function()
    local app, fake = shown()
    local notes = buffer.capture_notes()
    written(fake, fixtures.refusal("error-edit-due.json", EXIT_USAGE), 2, "due: the twelfth of never")
    local shown_errors = vim.diagnostic.get(0)
    assert(#shown_errors == 1 and shown_errors[1].lnum == 1, vim.inspect(shown_errors))
    assert(shown_errors[1].severity == vim.diagnostic.severity.ERROR)
    assert(shown_errors[1].message:find("--due cannot be", 1, true), shown_errors[1].message)
    assert(vim.bo.modified, "no longer modified")
    vim.wait(20)
    assert(#notes == 0 and #app.say:log() == 1, vim.inspect({ notes, app.say:log() }))
  end,

  ["a refusal that names no flag lands on line 1, and the next write clears it"] = function()
    local _, fake = shown()
    written(fake, fixtures.refusal("error-refused-edit.json", EXIT_REFUSED), 2, "due: 2026-10-02")
    assert(vim.diagnostic.get(0)[1].lnum == 0, vim.inspect(vim.diagnostic.get(0)))
    written(fake, fixtures.answer("edit.json"), 2, "due: 2026-10-03")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) == 0
    end)
  end,
}
