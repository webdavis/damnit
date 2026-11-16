local columns = require("damnit.list.columns")
local fixtures = require("helpers.fixtures")
local tree = require("damnit.list.tree")

local function task(oid, fields, details)
  return vim.tbl_extend("keep", fields, {
    oid = oid,
    kind = "task",
    subject = oid,
    labels = {},
    category = {},
    depends = {},
    overdue = false,
    blocked = false,
    task = vim.tbl_extend("keep", details or {}, { done = false, priority = 4 }),
  })
end

local function texts(layout, n, kept)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, columns.chunks(layout, n, kept)))
end

local function all(layout)
  return columns.fit(vim.tbl_extend("force", layout.widths, { text = 0 }), 1000)
end

return {
  ["each column is padded to its widest value so the columns line up"] = function()
    local layout = columns.layout(tree.arranged(fixtures.decoded("list-tree.json").objects))
    local kept = all(layout)
    assert(vim.deep_equal(kept, { "category", "waits", "due" }), vim.inspect(kept))
    local rows = {}
    for n = 1, 6 do
      rows[n] = texts(layout, n, kept)
    end
    local expected = {
      "                                                   ",
      "                  waits on buy oat milk            ",
      "                                                   ",
      "  context=errand                         2026-09-25",
      "                                                   ",
      "                                         2026-09-30",
    }
    assert(vim.deep_equal(rows, expected), vim.inspect(rows))
  end,

  ["labels, category, estimate, deadline and due each take their own group"] = function()
    local object = task(
      "a",
      { labels = { "deep", "urgent" }, category = { effort = "deep" }, overdue = true },
      { estimate = 45, deadline = "2026-11-20", due = "2026-11-14" }
    )
    local layout = columns.layout({ { object = object, depth = 0 } })
    local chunks = columns.chunks(layout, 1, all(layout))
    local expected = {
      { "  " },
      { "#urgent", "DamLabel" },
      { "  " },
      { "effort=deep", "DamCategory" },
      { "  " },
      { "45m", "DamMeta" },
      { "  " },
      { "deadline 2026-11-20", "DamDue" },
      { "  " },
      { "2026-11-14", "DamOverdue" },
    }
    assert(vim.deep_equal(chunks, expected), vim.inspect(chunks))
  end,

  ["waits on names a listed task by subject and an unlisted one by short oid"] = function()
    local first = task("1111111111", { subject = "book venue" })
    local waiting = task("2222222222", { depends = { "1111111111", "9999999999" } })
    local layout = columns.layout({ { object = first, depth = 0 }, { object = waiting, depth = 0 } })
    local text = texts(layout, 2, { "waits" })
    assert(text == "  waits on book venue, 9999999", text)
  end,

  ["the days words show only when dam's answer holds days_until_due"] = function()
    local words = {}
    for _, days in ipairs({ -2, -1, 0, 1, 3 }) do
      local layout = columns.layout({ { object = task("a", {}, { days_until_due = days }), depth = 0 } })
      table.insert(words, vim.trim(texts(layout, 1, { "days" })))
    end
    assert(vim.deep_equal(words, { "2d late", "1d late", "today", "tomorrow", "in 3d" }), vim.inspect(words))
    local without = columns.layout({ { object = task("a", {}, { due = "2026-11-16" }), depth = 0 } })
    assert(without.widths.days == 0, vim.inspect(without.widths))
  end,

  ["an event's start sits in the due column"] = function()
    local event = task("e", { kind = "event", event = { start = "2026-09-25T09:00" } })
    event.task = nil
    local layout = columns.layout({ { object = event, depth = 0 } })
    assert(texts(layout, 1, { "due" }) == "  2026-09-25T09:00", texts(layout, 1, { "due" }))
  end,

  ["fit keeps every column that has a value when the window is wide enough"] = function()
    local widths = { text = 30, labels = 7, category = 11, waits = 0, estimate = 3, deadline = 10, days = 0, due = 10 }
    local kept = columns.fit(widths, 81)
    assert(vim.deep_equal(kept, { "labels", "category", "estimate", "deadline", "due" }), vim.inspect(kept))
  end,

  ["fit drops labels first, then category, then waits on, estimate and deadline, and keeps due"] = function()
    local widths = { text = 30, labels = 7, category = 11, waits = 12, estimate = 3, deadline = 10, days = 0, due = 10 }
    assert(vim.deep_equal(columns.fit(widths, 75), { "waits", "estimate", "deadline", "due" }))
    assert(vim.deep_equal(columns.fit(widths, 55), { "deadline", "due" }))
    assert(vim.deep_equal(columns.fit(widths, 20), { "due" }))
  end,
}
