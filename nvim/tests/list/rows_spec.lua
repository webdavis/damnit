local fixtures = require("helpers.fixtures")
local rows = require("damnit.list.rows")
local tree = require("damnit.list.tree")

local SIGNS = require("damnit.kernel.options").defaults().signs

local function rendered(objects)
  return rows.render(tree.arranged(objects), SIGNS)
end

local function signs_on(marks, line)
  local signs = {}
  for _, mark in ipairs(marks) do
    if mark.line == line and mark.sign then
      table.insert(signs, mark.sign .. " " .. mark.sign_group)
    end
  end
  return signs
end

local function groups_on(marks, line)
  local groups = {}
  for _, mark in ipairs(marks) do
    if mark.line == line and mark.group then
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
  ["a row's text is only its indent, checkbox, priority and subject"] = function()
    local lines = rendered(fixtures.decoded("list-tree.json").objects)
    local expected = {
      "[ ] call mom",
      "[ ] wait for the milk",
      "[x] water the plants",
      "[ ] p2 buy oat milk",
      "  [ ] pack the bag",
      "[ ] pay rent",
    }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["overdue and blocked are highlighted only when dam says so"] = function()
    local _, marks = rendered(fixtures.decoded("list-tree.json").objects)
    assert(vim.deep_equal(groups_on(marks, 0), {}), vim.inspect(marks))
    assert(vim.deep_equal(groups_on(marks, 1), { "DamBlocked" }), vim.inspect(marks))
    assert(vim.deep_equal(groups_on(marks, 3), { "DamOverdue", "DamPriority" }), vim.inspect(marks))
    local past_due_but_not_overdue = task("a", { task = { done = false, priority = 4, due = "2000-01-01" } })
    local _, none = rendered({ past_due_but_not_overdue })
    assert(#none == 0, vim.inspect(none))
  end,

  ["overdue and blocked rows carry the signs from the options"] = function()
    local _, marks = rendered(fixtures.decoded("list-tree.json").objects)
    assert(vim.deep_equal(signs_on(marks, 1), { "~ DamBlocked" }), vim.inspect(marks))
    assert(vim.deep_equal(signs_on(marks, 3), { "! DamOverdue" }), vim.inspect(marks))
    assert(vim.deep_equal(signs_on(marks, 0), {}), vim.inspect(marks))
    local _, own = rows.render(tree.arranged(fixtures.decoded("list.json").objects), { overdue = "x", blocked = "b" })
    assert(vim.deep_equal(signs_on(own, 1), { "x DamOverdue" }), vim.inspect(own))
  end,

  ["the priority word takes the priority colour"] = function()
    local lines, marks = rendered(fixtures.decoded("list.json").objects)
    local priority = vim.tbl_filter(function(mark)
      return mark.group == "DamPriority"
    end, marks)
    assert(#priority == 1 and lines[2]:sub(priority[1].col + 1, priority[1].end_col) == "p2", vim.inspect(priority))
  end,

  ["a done task shows checked"] = function()
    local lines, marks = rendered(fixtures.decoded("list-tree.json").objects)
    assert(lines[3] == "[x] water the plants", lines[3])
    assert(vim.deep_equal(groups_on(marks, 2), { "DamDone" }), vim.inspect(marks))
  end,

  ["an event row shows its subject alone"] = function()
    local event = task("e", { kind = "event", subject = "standup", event = { start = "2026-09-25T09:00:00+00:00" } })
    event.task = nil
    local lines = rendered({ event })
    assert(vim.deep_equal(lines, { "standup" }), vim.inspect(lines))
  end,

  ["a mark covers the row after its indent"] = function()
    local lines, marks = rendered(fixtures.decoded("list.json").objects)
    local overdue = vim.tbl_filter(function(mark)
      return mark.group == "DamOverdue"
    end, marks)[1]
    assert(overdue.line == 1 and overdue.col == 0 and overdue.end_col == #lines[2], vim.inspect(marks))
  end,
}
