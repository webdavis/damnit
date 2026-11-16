local card = require("damnit.peek.card")
local fixtures = require("helpers.fixtures")

local function related()
  return fixtures.decoded("show-related.json")
end

local function texts(chunks)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, chunks))
end

return {
  ["the card is titled with the subject and lists each header field that holds a value"] = function()
    local shown = card.render(related(), 80)
    assert(shown.title == "plan the garden", shown.title)
    local expected = {
      "due         2026-09-25",
      "deadline    2026-09-30",
      "priority    p2",
      "labels      deep, garden",
      "category    effort=deep",
      "estimate    45m",
      "recurrence  every week",
      "path        call mom/",
      "parent      2222222",
    }
    assert(vim.deep_equal(vim.list_slice(shown.lines, 1, #expected), expected), vim.inspect(shown.lines))
  end,

  ["a field with no value and the hidden p4 are left out"] = function()
    local object = fixtures.decoded("list.json").objects[1]
    local shown = card.render(object, 80)
    assert(vim.deep_equal(shown.lines, {}), vim.inspect(shown.lines))
  end,

  ["the due carries dam's days words when dam counted them"] = function()
    local object = related()
    object.days_until_due = -2
    assert(card.render(object, 80).lines[1] == "due         2026-09-25 · 2d late", card.render(object, 80).lines[1])
    object.days_until_due = 3
    assert(card.render(object, 80).lines[1] == "due         2026-09-25 · in 3d")
  end,

  ["an event shows its start, end and location"] = function()
    local shown = card.render(fixtures.decoded("show-event.json"), 80)
    local expected = {
      "start     2026-09-25T09:00:00+00:00",
      "end       2026-09-25T09:15:00+00:00",
      "location  room 4",
      "path      work/",
    }
    assert(vim.deep_equal(shown.lines, expected), vim.inspect(shown.lines))
  end,

  ["a rule parts the fields from the body, cut to a dozen lines and a line counting the rest"] = function()
    local shown = card.render(related(), 80)
    assert(shown.lines[10] == ("─"):rep(shown.width), shown.lines[10])
    assert(shown.lines[11] == "step 1 of the plan", shown.lines[11])
    assert(shown.lines[22] == "step 12 of the plan", shown.lines[22])
    assert(shown.lines[23] == "… 8 more lines", shown.lines[23])
    assert(#shown.lines == 23, #shown.lines)
  end,

  ["a short body is shown whole"] = function()
    local shown = card.render(fixtures.decoded("show.json"), 80)
    assert(vim.deep_equal(vim.list_slice(shown.lines, #shown.lines - 1), { "loosen the soil", "water after" }))
  end,

  ["the footer holds the short oid and what dam flagged"] = function()
    local object = related()
    object.blocked = true
    assert(
      texts(card.render(object, 80).footer) == " <id-1> · overdue · blocked ",
      texts(card.render(object, 80).footer)
    )
    local overdue = card.render(object, 80).footer[3]
    assert(overdue[2] == "DamOverdue", vim.inspect(overdue))
  end,

  ["the card is as wide as its widest line, up to the room it is given"] = function()
    local shown = card.render(related(), 80)
    assert(shown.width == 24, shown.width)
    local object = related()
    object.body = ("word "):rep(40)
    assert(card.render(object, 60).width == 60)
    assert(card.render(object, 30).width == 30)
  end,

  ["the field names are marked as meta"] = function()
    local shown = card.render(related(), 80)
    assert(
      vim.deep_equal(shown.marks[1], { line = 0, col = 0, end_col = 3, group = "DamMeta" }),
      vim.inspect(shown.marks[1])
    )
  end,
}
