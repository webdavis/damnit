local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local EXIT_USAGE, EXIT_REFUSED = 2, 4

return {
  [":Dam messages shows each error with its suggested command"] = function()
    local app, fake = helper.new()
    require("damnit.command").define(app, { require("damnit.messages").new(app) })
    buffer.capture_notes()
    fake.reply("list", fixtures.refusal("error-usage-selection.json", EXIT_USAGE))
    fake.reply("show", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    local first = helper.call(app, { args = { "list", "--priority", "urgent" }, cwd = "/" }, "write")
    local second = helper.call(app, { args = { "show", "0000000" }, cwd = "/" }, "write")
    buffer.wait_for(function()
      return first.done and second.done
    end)
    vim.cmd("Dam messages")
    local shown = table.concat(buffer.lines(), "\n")
    assert(vim.bo.filetype == "dammessages", vim.bo.filetype)
    assert(shown:find("usage: --priority cannot be", 1, true), shown)
    assert(shown:find("dam list --priority 1", 1, true), shown)
    assert(shown:find("refused no_such_object: no object matches", 1, true), shown)
  end,

  [":Dam messages says when there is nothing to show"] = function()
    local app = helper.new()
    require("damnit.command").define(app, { require("damnit.messages").new(app) })
    vim.cmd("Dam messages")
    assert(vim.deep_equal(buffer.lines(), { "no dam errors yet" }), vim.inspect(buffer.lines()))
  end,

  ["a suggested command the message leaves out gets its own run line"] = function()
    local lines = require("damnit.messages").lines({
      {
        kind = "usage",
        message = '--due cannot be "someday"',
        oids = {},
        selection = { command = "dam list --due today" },
      },
    })
    assert(
      vim.deep_equal(lines, { 'usage: --due cannot be "someday"', "run: dam list --due today" }),
      vim.inspect(lines)
    )
  end,
}
