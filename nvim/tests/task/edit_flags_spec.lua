local edit_flags = require("damnit.task.edit_flags")

local function task_fields(changes)
  return vim.tbl_extend("force", {
    subject = "plant the tomatoes",
    due = "2026-09-25",
    deadline = "2026-09-30",
    priority = "2",
    labels = { "garden" },
    category = { effort = { "deep" } },
    estimate = "45",
    recurrence = "every week",
    body = "loosen the soil",
  }, changes or {})
end

local function event_fields(changes)
  return vim.tbl_extend("force", {
    subject = "standup",
    start = "2026-09-25T09:00:00+00:00",
    ["end"] = "2026-09-25T09:15:00+00:00",
    location = "room 4",
    labels = {},
    category = {},
    body = "",
  }, changes or {})
end

local function sent(after, before)
  return edit_flags.diff(before or task_fields(), after)
end

local function equal(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["an untouched header sends no flags"] = function()
    equal(sent(task_fields()), {})
  end,

  ["changing the due line sends --due with the typed text"] = function()
    equal(sent(task_fields({ due = "next friday" })), { "--due", "next friday" })
  end,

  ["clearing the due line sends --no-due"] = function()
    equal(sent(task_fields({ due = "" })), { "--no-due" })
  end,

  ["each cleared field sends its own clearing flag"] = function()
    local cleared = task_fields({ deadline = "", estimate = "", recurrence = "" })
    equal(sent(cleared), { "--no-deadline", "--no-estimate", "--no-recurrence" })
  end,

  ["each changed field sends its flag with the typed text"] = function()
    local changed =
      task_fields({ subject = "plant", deadline = "fri", priority = "x", estimate = "1h", recurrence = "daily" })
    equal(sent(changed), {
      "--subject",
      "plant",
      "--deadline",
      "fri",
      "--priority",
      "x",
      "--estimate",
      "1h",
      "--recurrence",
      "daily",
    })
  end,

  ["a removed label is sent as --unlabels and an added one as --labels"] = function()
    equal(sent(task_fields({ labels = { "seeds" } })), { "--labels", "seeds", "--unlabels", "garden" })
  end,

  ["a cleared category is sent as G=none"] = function()
    equal(sent(task_fields({ category = {} })), { "--category", "effort=none" })
  end,

  ["a changed category label is added and the old one taken off"] = function()
    local after = task_fields({ category = { effort = { "light" } } })
    equal(sent(after), { "--category", "effort=light", "--unlabels", "deep" })
  end,

  ["a changed body is sent as --body"] = function()
    equal(sent(task_fields({ body = "water first\nthen plant" })), { "--body", "water first\nthen plant" })
  end,

  ["an event sends --start, --end and --location, and --no-location when cleared"] = function()
    local after = event_fields({ start = "tomorrow 10:00", ["end"] = "tomorrow 11:00", location = "" })
    equal(
      edit_flags.diff(event_fields(), after),
      { "--start", "tomorrow 10:00", "--end", "tomorrow 11:00", "--no-location" }
    )
    local moved = event_fields({ location = "room 5" })
    equal(edit_flags.diff(event_fields(), moved), { "--location", "room 5" })
  end,
}
