local fixtures = require("helpers.fixtures")
local rows = require("damnit.list.rows")
local tree = require("damnit.list.tree")

local function rendered(objects)
  return rows.render(tree.arranged(objects))
end

local function groups_on(marks, line)
  local groups = {}
  for _, mark in ipairs(marks) do
    if mark.line == line then
      table.insert(groups, mark.group)
    end
  end
  table.sort(groups)
  return groups
end

local function task(oid, fields)
  return vim.tbl_extend("keep", fields, {
    oid = oid,
    kind = "task",
    subject = oid,
    labels = {},
    category = {},
    overdue = false,
    blocked = false,
    task = { done = false, priority = 4 },
  })
end

return {
  ["rows show subject, priority, due and labels in dam's order"] = function()
    local lines = rendered(fixtures.decoded("list-tree.json").objects)
    local expected = {
      "[ ] call mom",
      "[ ] wait for the milk",
      "[x] water the plants",
      "[ ] p2 buy oat milk 2026-09-25 context=errand",
      "  [ ] pack the bag",
      "[ ] pay rent 2026-09-30",
    }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["overdue and blocked are highlighted only when dam says so"] = function()
    local _, marks = rendered(fixtures.decoded("list-tree.json").objects)
    assert(vim.deep_equal(groups_on(marks, 0), {}), vim.inspect(marks))
    assert(vim.deep_equal(groups_on(marks, 1), { "DamBlocked" }), vim.inspect(marks))
    assert(vim.deep_equal(groups_on(marks, 3), { "DamOverdue" }), vim.inspect(marks))
    local past_due_but_not_overdue = task("a", { task = { done = false, priority = 4, due = "2000-01-01" } })
    local _, none = rendered({ past_due_but_not_overdue })
    assert(#none == 0, vim.inspect(none))
  end,

  ["a label that is a category's value shows only as group=label"] = function()
    local object = task("a", { labels = { "deep", "urgent" }, category = { effort = "deep" } })
    local lines = rendered({ object })
    assert(vim.deep_equal(lines, { "[ ] a #urgent effort=deep" }), vim.inspect(lines))
  end,

  ["a done task shows checked"] = function()
    local lines, marks = rendered(fixtures.decoded("list-tree.json").objects)
    assert(lines[3] == "[x] water the plants", lines[3])
    assert(vim.deep_equal(groups_on(marks, 2), { "DamDone" }), vim.inspect(marks))
  end,

  ["an event row shows its start"] = function()
    local event = task("e", { kind = "event", subject = "standup", event = { start = "2026-09-25T09:00:00+00:00" } })
    event.task = nil
    local lines = rendered({ event })
    assert(vim.deep_equal(lines, { "standup 2026-09-25T09:00:00+00:00" }), vim.inspect(lines))
  end,

  ["a mark covers the row after its indent"] = function()
    local lines, marks = rendered(fixtures.decoded("list.json").objects)
    local overdue = marks[1]
    assert(overdue.line == 1 and overdue.col == 0 and overdue.end_col == #lines[2], vim.inspect(marks))
  end,
}
