local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local today = require("helpers.today")

local function done_calls(fake)
  return vim.tbl_filter(function(call)
    return call.argv[2] == "done"
  end, fake.calls)
end

return {
  ["x on a header line warns and sends nothing"] = function()
    local _, fake = today.opened()
    local notes = buffer.capture_notes()
    today.on("Today (1)")
    buffer.feed("x")
    vim.wait(30)
    assert(#done_calls(fake) == 0, vim.inspect(done_calls(fake)))
    assert(#notes == 1 and notes[1].level == vim.log.levels.WARN, vim.inspect(notes))
  end,

  ["[count] rows from the cursor leave the header lines out"] = function()
    local _, fake = today.opened()
    fake.reply("done", function(argv)
      return fixtures.answer(vim.tbl_contains(argv, "--preview") and "done-preview.json" or "done.json")
    end)
    today.on("[ ] call the bank")
    buffer.feed("3x")
    buffer.wait_for(function()
      return #done_calls(fake) == 1
    end)
    local call = done_calls(fake)[1]
    local oids = vim.split(vim.trim(call.stdin), "\n")
    local want = { "--expect", "2", "--preview" }
    assert(vim.deep_equal(vim.list_slice(call.argv, 4, 6), want), vim.inspect(call.argv))
    assert(#oids == 2, vim.inspect(oids))
    buffer.wait_for(function()
      return require("helpers.list").confirm_window() ~= nil
    end)
    buffer.feed("n")
  end,

  ["a change prompt from a sectioned view reads the rows under the headers"] = function()
    local _, fake = today.opened()
    fake.reply("edit", fixtures.answer("edit-preview-one.json"))
    for _, verb in ipairs({ "filter", "category", "remote", "label", "path" }) do
      fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", 2))
    end
    today.on("[ ] call the bank")
    require("helpers.prompt").type("cd2026-11-20")
    require("helpers.prompt").footer_reads("due  none → 2026-11-20")
  end,
}
