local fixtures = require("helpers.fixtures")
local sections = require("damnit.list.sections")

local function shown(rows)
  return vim.tbl_map(function(row)
    return row.header or (("  "):rep(row.depth) .. row.object.subject)
  end, rows)
end

local function segment(title, name)
  return { title = title, objects = fixtures.decoded(name).objects }
end

return {
  ["each section's rows sit under a header naming its title and count"] = function()
    local rows = sections.rows({
      segment("Overdue", "today-overdue.json"),
      segment("Today", "today-today.json"),
      { title = "Next 7 days", objects = {} },
    })
    local expected = { "Overdue (1)", "renew the passport", "Today (1)", "call the bank", "Next 7 days (0)" }
    assert(vim.deep_equal(shown(rows), expected), vim.inspect(shown(rows)))
  end,

  ["header rows carry no object"] = function()
    local rows = sections.rows({ segment("Today", "today-today.json") })
    assert(rows[1].object == nil and rows[1].depth == 0, vim.inspect(rows[1]))
  end,

  ["the tree is drawn inside a section"] = function()
    local rows = sections.rows({ segment("No date", "today-none.json") })
    local expected = { "No date (3)", "plan the move", "  pack the books", "read the lease" }
    assert(vim.deep_equal(shown(rows), expected), vim.inspect(shown(rows)))
  end,

  ["a child whose parent is in another section is a root there"] = function()
    local rows = sections.rows({ segment("Next 7 days", "today-next.json"), segment("No date", "today-none.json") })
    assert(rows[3].object.subject == "book the van" and rows[3].depth == 0, vim.inspect(shown(rows)))
  end,

  ["a failed section shows dam's message on its header and the others still draw"] = function()
    local refused = fixtures.decoded("error-usage-selection.json").error
    local rows = sections.rows({ { title = "Overdue", error = refused }, segment("Today", "today-today.json") })
    assert(rows[1].header == "Overdue: " .. refused.message and rows[1].failed, vim.inspect(rows[1]))
    assert(vim.deep_equal(shown(rows), { rows[1].header, "Today (1)", "call the bank" }), vim.inspect(shown(rows)))
  end,
}
