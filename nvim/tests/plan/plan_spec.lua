local buffer = require("helpers.buffer")
local plan = require("helpers.plan")

local LINES = {
  "Unplanned (1)",
  "[ ] read the lease",
  "today · <today> (1)",
  "[ ] pay rent",
  "today+1 · <today+1> (1)",
  "[ ] call the bank",
  "today+2 (0)",
  "today+3 (0)",
  "today+4 (0)",
  "today+5 (0)",
  "today+6 (0)",
}

local function edit_of(oid, ...)
  return vim.list_extend({ "dam", "edit", oid, ... }, { "-o", "json" })
end

return {
  [":Dam plan sends one dam list --request - per section"] = function()
    local _, fake = plan.opened()
    local days = {}
    for _, call in ipairs(fake.of("list")) do
      assert(vim.deep_equal(call.argv, { "dam", "list", "--request", "-", "-o", "json" }), vim.inspect(call.argv))
      table.insert(days, vim.json.decode(call.stdin)["focus.day"])
    end
    local expected = { "none", "today", "today+1", "today+2", "today+3", "today+4", "today+5", "today+6" }
    assert(vim.deep_equal(days, expected), vim.inspect(days))
    local unplanned = vim.json.decode(fake.of("list")[1].stdin)
    assert(unplanned.blocked == false and unplanned.limit == 50, vim.inspect(unplanned))
  end,

  ["each section's rows sit under its anchor word and the day dam printed for its marks"] = function()
    plan.opened()
    assert(vim.deep_equal(buffer.lines(), LINES), vim.inspect(buffer.lines()))
  end,

  ["> on an unplanned task marks it for today"] = function()
    local _, fake = plan.opened()
    plan.on("[ ] read the lease")
    buffer.feed(">")
    local argv = plan.edited(fake, 1)
    assert(vim.deep_equal(argv, edit_of("none-1", "--focus", "today")), vim.inspect(argv))
  end,

  ["> on today's task moves it to tomorrow and < clears it"] = function()
    local _, fake = plan.opened()
    plan.on("[ ] pay rent")
    buffer.feed(">")
    local argv = plan.edited(fake, 1)
    assert(vim.deep_equal(argv, edit_of("today-1", "--focus", "today+1")), vim.inspect(argv))
    plan.on("[ ] pay rent")
    buffer.feed("<lt>")
    argv = plan.edited(fake, 2)
    assert(vim.deep_equal(argv, edit_of("today-1", "--no-focus")), vim.inspect(argv))
  end,

  ["< on an unplanned task warns and sends nothing"] = function()
    local _, fake = plan.opened()
    local notes = buffer.capture_notes()
    plan.on("[ ] read the lease")
    buffer.feed("<lt>")
    assert(#fake.of("edit") == 0 and notes[1] and notes[1].text:find("not planned"), vim.inspect(notes))
  end,

  ["> over rows of two sections warns and sends nothing"] = function()
    local _, fake = plan.opened()
    local notes = buffer.capture_notes()
    plan.on("[ ] read the lease")
    buffer.feed("Vjj>")
    assert(#fake.of("edit") == 0 and notes[1] and notes[1].text:find("one section"), vim.inspect(notes))
  end,

  ["dam's refusal of a full day shows on the pressed row and on the day's marks, and nothing redraws"] = function()
    local _, fake = plan.opened(true)
    plan.on("[ ] read the lease")
    buffer.feed(">")
    plan.edited(fake, 1, true)
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    local message = vim.diagnostic.get(0)[1].message
    assert(message:find("already has 3 focus tasks", 1, true), message)
    local rows = vim.tbl_map(function(diagnostic)
      return LINES[diagnostic.lnum + 1]
    end, vim.diagnostic.get(0))
    table.sort(rows)
    assert(vim.deep_equal(rows, { "[ ] pay rent", "[ ] read the lease" }), vim.inspect(rows))
    assert(vim.deep_equal(buffer.lines(), LINES), vim.inspect(buffer.lines()))
  end,
}
