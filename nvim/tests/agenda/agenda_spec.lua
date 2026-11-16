local agenda = require("helpers.agenda")
local buffer = require("helpers.buffer")
local grid = require("helpers.grid")

local LINES = {
  "Overdue (1)",
  "[ ] renew the passport",
  "tue 2030-01-08",
  "all day      offsite",
  "[ ] pay rent",
  "wed 2030-01-09",
  "09:00–09:30  standup",
  "thu 2030-01-10",
  "[ ] call the bank",
}

return {
  [":Dam agenda asks dam for this week's events and tasks, and for the overdue tasks"] = function()
    local _, fake = agenda.opened()
    local sent = fake.of("agenda")[1].argv
    local expected = { "dam", "agenda", "--kind", "any", "--from", "week", "--to", "week", "-o", "json" }
    assert(vim.deep_equal(sent, expected), vim.inspect(sent))
    local overdue = fake.of("list")[1].argv
    assert(vim.deep_equal(overdue, { "dam", "list", "--overdue", "-o", "json" }), vim.inspect(overdue))
    assert(vim.api.nvim_buf_get_name(0) == "dam://agenda", vim.api.nvim_buf_get_name(0))
    assert(vim.bo.filetype == "damagenda", vim.bo.filetype)
  end,

  [":Dam agenda day asks for today alone"] = function()
    local _, fake = agenda.opened("Dam agenda day")
    assert(vim.deep_equal(agenda.last_window(fake), { "today", "today" }), vim.inspect(agenda.last_window(fake)))
  end,

  ["the overdue section comes first, then each day's events and then its tasks"] = function()
    agenda.opened()
    assert(vim.deep_equal(buffer.lines(), LINES), vim.inspect(buffer.lines()))
  end,

  ["the winbar names the window dam answered for and counts its events and tasks"] = function()
    agenda.opened()
    assert(grid.row(1):find("agenda · 2030-01-07 … 2030-01-13 · 2 events · 2 due", 1, true), grid.row(1))
  end,

  ["an event row shows busy at the right and its path stays out when every row shares one"] = function()
    agenda.opened()
    agenda.on("09:00–09:30  standup")
    local row = grid.row(vim.fn.line(".") + 1)
    assert(row:find("standup%s+busy%s*$"), row)
  end,

  ["> asks for next week without the overdue section, < for the week before, and T for this week"] = function()
    local _, fake = agenda.opened()
    buffer.feed(">")
    buffer.wait_for(function()
      return agenda.last_window(fake)[1] == "week+1" and buffer.lines()[1] ~= "Overdue (1)"
    end)
    assert(#fake.of("list") == 1, vim.inspect(fake.of("list")))
    buffer.feed("<lt>")
    buffer.feed("<lt>")
    buffer.wait_for(function()
      return agenda.last_window(fake)[1] == "week-1"
    end)
    buffer.feed("T")
    buffer.wait_for(function()
      return agenda.last_window(fake)[1] == "week" and buffer.lines()[1] == "Overdue (1)"
    end)
  end,

  ["x on a task row sends dam done for it"] = function()
    local _, fake = agenda.opened()
    agenda.on("[ ] pay rent")
    buffer.feed("x")
    buffer.wait_for(function()
      return #fake.of("done") == 1 and #fake.of("agenda") == 2
    end)
    local argv = fake.of("done")[1].argv
    assert(argv[2] == "done" and argv[3] ~= nil and argv[3] ~= "-", vim.inspect(argv))
  end,

  ["x on an event row warns and sends nothing"] = function()
    local _, fake = agenda.opened()
    local notes = buffer.capture_notes()
    agenda.on("all day      offsite")
    buffer.feed("x")
    assert(#fake.of("done") == 0, vim.inspect(fake.of("done")))
    assert(notes[1] and notes[1].text:find("no task", 1, true), vim.inspect(notes))
  end,

  ["<CR> on an event row opens it"] = function()
    local _, _, opened = agenda.opened()
    agenda.on("09:00–09:30  standup")
    buffer.feed("<CR>")
    assert(#opened == 1 and #opened[1] > 0, vim.inspect(opened))
  end,

  ["a change elsewhere reads the agenda again"] = function()
    local _, fake = agenda.opened()
    vim.api.nvim_exec_autocmds("User", { pattern = "DamChanged" })
    buffer.wait_for(function()
      return #fake.of("agenda") == 2
    end)
  end,

  ["a day header folds with <Tab>"] = function()
    agenda.opened()
    agenda.on("tue 2030-01-08")
    buffer.feed("<Tab>")
    assert(vim.fn.foldclosed(3) == 3 and vim.fn.foldclosedend(3) == 5, vim.fn.foldclosedend(3))
  end,

  ["f in the agenda filters with dam list and draws its rows as a list"] = function()
    local prompt = require("helpers.prompt")
    local _, fake = agenda.opened()
    local agenda_buf = vim.api.nvim_get_current_buf()
    buffer.feed("f")
    prompt.footer_reads("1 row")
    assert(
      vim.deep_equal(buffer.lines(agenda_buf), { "[ ] renew the passport" }),
      vim.inspect(buffer.lines(agenda_buf))
    )
    assert(#fake.of("list") >= 2, vim.inspect(fake.of("list")))
  end,
}
