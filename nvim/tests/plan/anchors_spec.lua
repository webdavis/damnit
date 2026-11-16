local anchors = require("damnit.plan.anchors")
local fixtures = require("helpers.fixtures")

local function rows(...)
  local found = {}
  for _, section in ipairs({ ... }) do
    table.insert(found, { header = section[1], key = section[1], depth = 0 })
    for _, oid in ipairs(section[2]) do
      table.insert(found, { object = { oid = oid }, depth = 0 })
    end
  end
  return found
end

return {
  ["the sections are the unplanned open tasks that are not blocked, then today and the six days after it"] = function()
    local sections = anchors.sections()
    local titles = vim.tbl_map(function(section)
      return section.title
    end, sections)
    local expected = { "Unplanned", "today", "today+1", "today+2", "today+3", "today+4", "today+5", "today+6" }
    assert(vim.deep_equal(titles, expected), vim.inspect(titles))
    local unplanned = { title = "Unplanned", ["focus.day"] = "none", blocked = false, limit = anchors.LIMIT }
    assert(vim.deep_equal(sections[1], unplanned), vim.inspect(sections[1]))
    assert(vim.deep_equal(sections[3], { title = "today+1", ["focus.day"] = "today+1" }), vim.inspect(sections[3]))
  end,

  ["a day's header shows the day dam printed for the focus mark"] = function()
    local objects = fixtures.decoded("plan-today+1.json").objects
    assert(anchors.header("today+1", objects) == "today+1 · <today+1> (1)", anchors.header("today+1", objects))
  end,

  ["a day with no rows shows the anchor word and its count"] = function()
    assert(anchors.header("today+2", {}) == "today+2 (0)", anchors.header("today+2", {}))
  end,

  ["the unplanned header says when dam's limit cut the rows"] = function()
    local objects = {}
    for n = 1, anchors.LIMIT do
      objects[n] = { oid = tostring(n) }
    end
    local header = anchors.header("Unplanned", objects)
    assert(header == ("Unplanned (first %d)"):format(anchors.LIMIT), header)
    assert(anchors.header("Unplanned", { {} }) == "Unplanned (1)", anchors.header("Unplanned", { {} }))
  end,

  ["> sets the focus a day later and < a day earlier, clearing it from today"] = function()
    local moves = {
      anchors.moved("Unplanned", 1),
      anchors.moved("today", 1),
      anchors.moved("today+1", 1),
      anchors.moved("today+6", 1),
      anchors.moved("today+2", -1),
      anchors.moved("today+1", -1),
      anchors.moved("today", -1),
    }
    local expected = {
      "--focus today",
      "--focus today+1",
      "--focus today+2",
      "--focus today+7",
      "--focus today+1",
      "--focus today",
      "--no-focus",
    }
    assert(vim.deep_equal(moves, expected), vim.inspect(moves))
  end,

  ["< on an unplanned task has nowhere to go"] = function()
    local flags, problem = anchors.moved("Unplanned", -1)
    assert(flags == nil and problem:find("not planned"), problem)
  end,

  ["the section of the targets is the header above them"] = function()
    local drawn = rows({ "Unplanned", { "a", "b" } }, { "today", { "c" } })
    assert(anchors.section_of(drawn, { "a", "b" }) == "Unplanned")
    assert(anchors.section_of(drawn, { "c" }) == "today")
  end,

  ["targets in two sections have no one section"] = function()
    local drawn = rows({ "Unplanned", { "a" } }, { "today", { "c" } })
    assert(anchors.section_of(drawn, { "a", "c" }) == nil)
  end,
}
