local fixtures = require("helpers.fixtures")
local header = require("damnit.task.header")

local TASK_LINES = {
  "subject: plant the tomatoes",
  "due: 2026-09-25",
  "deadline: 2026-09-30",
  "priority: 2",
  "labels: garden",
  "category: effort=deep",
  "estimate: 45",
  "recurrence: every week",
  "---",
  "loosen the soil",
  "water after",
}

local function equal(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

local function with_line(n, text)
  local lines = vim.list_slice(TASK_LINES)
  lines[n] = text
  return lines
end

local function problem_of(lines)
  local fields, problem = header.parse(lines)
  assert(fields == nil, vim.inspect(fields))
  return problem
end

return {
  ["the header shows each field dam printed"] = function()
    local lines = header.lines(fixtures.decoded("show.json"))
    equal(lines, TASK_LINES)
  end,

  ["the read-only facts are not in the buffer lines"] = function()
    for _, line in ipairs(header.lines(fixtures.decoded("show.json"))) do
      assert(not line:find("^oid:") and not line:find("^path:") and not line:find("^overdue:"), line)
    end
  end,

  ["an event shows start, end and location in place of the task's dates"] = function()
    local lines = header.lines(fixtures.decoded("show-event.json"))
    equal(lines, {
      "subject: standup",
      "start: 2026-09-25T09:00:00+00:00",
      "end: 2026-09-25T09:15:00+00:00",
      "location: room 4",
      "labels:",
      "category:",
      "---",
    })
  end,

  ["a group holding several labels shows each as group=label"] = function()
    local object = fixtures.decoded("show.json")
    object.labels = { "errand", "garden", "home" }
    object.category = { context = { "errand", "home" } }
    local lines = header.lines(object)
    equal({ lines[5], lines[6] }, { "labels: garden", "category: context=errand, context=home" })
  end,

  ["parse reads back the fields the header shows"] = function()
    local fields = assert(header.parse(TASK_LINES))
    equal(fields.due, "2026-09-25")
    equal(fields.labels, { "garden" })
    equal(fields.category, { effort = { "deep" } })
    equal(fields.body, "loosen the soil\nwater after")
  end,

  ["parse names a line that is not a header field"] = function()
    local problem = problem_of(with_line(4, "urgency: high"))
    assert(problem:find("line 4", 1, true) and problem:find("urgency", 1, true), problem)
  end,

  ["parse names a field given twice"] = function()
    local problem = problem_of(with_line(3, "due: tomorrow"))
    assert(problem:find("line 3", 1, true) and problem:find("due", 1, true), problem)
  end,

  ["parse names a missing field"] = function()
    local lines = vim.list_slice(TASK_LINES)
    table.remove(lines, 2)
    local problem = problem_of(lines)
    assert(problem:find("due:", 1, true), problem)
  end,

  ["parse names a category that is not group=label"] = function()
    local problem = problem_of(with_line(6, "category: deep"))
    assert(problem:find("line 6", 1, true) and problem:find("group=label", 1, true), problem)
  end,

  ["parse asks for the --- line under the header"] = function()
    local problem = problem_of(vim.list_slice(TASK_LINES, 1, 8))
    assert(problem:find("---", 1, true), problem)
  end,
}
