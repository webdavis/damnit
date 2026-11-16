local facts = require("damnit.task.facts")
local fixtures = require("helpers.fixtures")

local function equal(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["the winbar facts are the short oid, path, author, parent and what dam flagged"] = function()
    local object = fixtures.decoded("show.json")
    object.parent = "1111111111111111111111111111111111111111"
    object.blocked = true
    equal(facts.chunks(object), {
      { " <id-1>", "DamMeta" },
      { " · " },
      { "home/" },
      { "", cut = true },
      { " · " },
      { "by owner" },
      { " · " },
      { "parent 1111111" },
      { " · " },
      { "overdue", "DamOverdue" },
      { " · " },
      { "blocked", "DamBlocked" },
    })
  end,

  ["a task at the root with no parent shows no path or parent"] = function()
    local object = fixtures.decoded("show.json")
    object.path, object.overdue = "", false
    equal(facts.chunks(object), { { " <id-1>", "DamMeta" }, { "", cut = true }, { " · " }, { "by owner" } })
  end,
}
