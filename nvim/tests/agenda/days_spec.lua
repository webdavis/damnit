local days = require("damnit.agenda.days")
local fixtures = require("helpers.fixtures")

local function shown(segments)
  local lines = {}
  for _, segment in ipairs(segments) do
    table.insert(lines, segment.title)
    for _, row in ipairs(segment.rows) do
      table.insert(lines, (row.lead and (row.lead .. "  ") or "") .. row.object.subject)
    end
  end
  return lines
end

local function text_of(chunks)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, chunks))
end

return {
  ["each day that holds a row is a header of its weekday and day, its events first and then its tasks"] = function()
    local segments = days.segments(fixtures.decoded("agenda-next-week.json"))
    local expected = {
      "tue 2030-01-08",
      "all day      offsite",
      "pay rent",
      "wed 2030-01-09",
      "09:00–09:30  standup",
      "thu 2030-01-10",
      "call the bank",
    }
    assert(vim.deep_equal(shown(segments), expected), vim.inspect(shown(segments)))
  end,

  ["a day's header carries no count"] = function()
    local segments = days.segments(fixtures.decoded("agenda-next-week.json"))
    assert(segments[1].uncounted == true, vim.inspect(segments[1]))
  end,

  ["an event row is read-only and a task row is not"] = function()
    local rows = days.segments(fixtures.decoded("agenda-next-week.json"))[1].rows
    assert(rows[1].readonly == true and rows[1].depth == 0, vim.inspect(rows[1]))
    assert(not rows[2].readonly and rows[2].object.kind == "task", vim.inspect(rows[2]))
  end,

  ["an answer with no rows has no days"] = function()
    assert(#days.segments(fixtures.decoded("agenda-week.json")) == 0)
  end,

  ["the winbar names the window dam answered for and counts the events and the tasks due"] = function()
    local text = text_of(days.bar(fixtures.decoded("agenda-next-week.json"), 0))
    assert(text == " agenda · 2030-01-07 … 2030-01-13 · 2 events · 2 due", text)
  end,

  ["the winbar counts the marked rows when there are some"] = function()
    local text = text_of(days.bar(fixtures.decoded("agenda-day.json"), 2))
    assert(text == " agenda · <today> … <today> · 1 event · 1 due · 2 marked", text)
  end,

  ["the window is dam's anchor word moved by the offset"] = function()
    local words = {
      days.window("week", 0),
      days.window("week", 1),
      days.window("week", -2),
      days.window("day", 0),
      days.window("day", 3),
      days.window("day", -1),
    }
    local expected = { "week", "week+1", "week-2", "today", "today+3", "today-1" }
    assert(vim.deep_equal(words, expected), vim.inspect(words))
  end,
}
