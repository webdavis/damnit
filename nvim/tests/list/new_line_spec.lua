local fixtures = require("helpers.fixtures")
local new_line = require("damnit.list.new_line")

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["the words before the first flag are dam new's one subject"] = function()
    same(new_line.args("buy oat milk --due fri --labels errand"), {
      "new",
      "buy oat milk",
      "--due",
      "fri",
      "--labels",
      "errand",
    })
  end,

  ["quotes hold a value's spaces"] = function()
    same(new_line.args([[buy "oat milk" --recurrence 'every 2 weeks']]), {
      "new",
      "buy oat milk",
      "--recurrence",
      "every 2 weeks",
    })
  end,

  ["a line with no subject sends none, so dam says it is missing"] = function()
    same(new_line.args("--parent 1111111"), { "new", "--parent", "1111111" })
  end,

  ["the reading is dam's subject, due, labels and path"] = function()
    local reading = new_line.reading(fixtures.decoded("new-preview.json"))
    assert(reading == 'task "buy oat milk" · due 2026-11-20 · errand · home/', reading)
  end,

  ["a reading leaves out what dam did not set"] = function()
    local reading = new_line.reading({
      changes = { { kind = "task", subject = "call mom", path = "", labels = {} } },
    })
    assert(reading == 'task "call mom"', reading)
  end,
}
