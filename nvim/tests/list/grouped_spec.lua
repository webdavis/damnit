local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local grid = require("helpers.grid")
local helper = require("helpers.app")

local function opened(view)
  local app, fake = helper.new({ views = { places = view } })
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", fixtures.answer("list-by-path.json"))
  vim.cmd("Dam list places")
  buffer.wait_for(function()
    return #buffer.lines() > 4
  end)
  return fake
end

return {
  ["a grouped view sends dam its flags without group and puts a header wherever the path changes"] = function()
    local fake = opened({ sort = "path", group = true })
    assert(vim.deep_equal(vim.json.decode(fake.of("list")[1].stdin), { sort = "path" }), fake.of("list")[1].stdin)
    local expected = {
      "none (1)",
      "[ ] call mom",
      "home/ (2)",
      "[ ] p2 buy oat milk",
      "[ ] pay rent",
      "work/ (2)",
      "[ ] file the report",
      "[ ] fix the gate",
    }
    assert(vim.deep_equal(buffer.lines(), expected), vim.inspect(buffer.lines()))
  end,

  ["a view grouped by path leaves the path column out"] = function()
    opened({ sort = "path", group = true })
    assert(not grid.row(5):find("home/", 1, true), grid.row(5))
  end,

  ["the same view without group has no headers and shows the path column"] = function()
    opened({ sort = "path" })
    assert(buffer.lines()[1] == "[ ] call mom", vim.inspect(buffer.lines()))
    assert(grid.row(3):find("home/", 1, true), grid.row(3))
  end,

  ["group without a sort draws no headers"] = function()
    opened({ group = true })
    assert(buffer.lines()[1] == "[ ] call mom", vim.inspect(buffer.lines()))
  end,
}
