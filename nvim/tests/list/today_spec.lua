local buffer = require("helpers.buffer")
local grid = require("helpers.grid")
local today = require("helpers.today")

local LINES = {
  "Overdue (1)",
  "[ ] renew the passport",
  "Today (1)",
  "[ ] call the bank",
  "Next 7 days (2)",
  "[ ] send the invoice",
  "[ ] book the van",
  "No date (3)",
  "[ ] plan the move",
  "  [ ] pack the books",
  "[ ] read the lease",
}

return {
  [":Dam today sends one dam list --request - per section"] = function()
    local _, fake = today.opened()
    local sent = {}
    for _, call in ipairs(fake.of("list")) do
      assert(vim.deep_equal(call.argv, { "dam", "list", "--request", "-", "-o", "json" }), vim.inspect(call.argv))
      sent[#sent + 1] = call.stdin
    end
    assert(#sent == 4, vim.inspect(sent))
    assert(vim.api.nvim_buf_get_name(0) == "dam://today", vim.api.nvim_buf_get_name(0))
  end,

  [":Dam today puts each task under its section's header"] = function()
    today.opened()
    assert(vim.deep_equal(buffer.lines(), LINES), vim.inspect(buffer.lines()))
  end,

  ["a header line is drawn in DamHeader"] = function()
    today.opened()
    local namespace = vim.api.nvim_get_namespaces()["damnit"]
    local marks = vim.api.nvim_buf_get_extmarks(0, namespace, { 7, 0 }, { 7, -1 }, { details = true })
    local groups = vim.tbl_map(function(mark)
      return mark[4].hl_group
    end, marks)
    assert(vim.tbl_contains(groups, "DamHeader"), vim.inspect(marks))
  end,

  ["a failed section shows dam's message on its header and the others still draw"] = function()
    today.opened({ ['{"due":"today","overdue":false}'] = true })
    local lines = buffer.lines()
    assert(lines[3]:find("^Today: %-%-priority cannot be"), vim.inspect(lines))
    assert(lines[4] == "Next 7 days (2)", vim.inspect(lines))
  end,

  ["the winbar counts the rows of every section"] = function()
    today.opened()
    assert(grid.row(1):find(" today · .* · 7 tasks · 1 overdue"), grid.row(1))
  end,

  ["<CR> on a header warns and opens nothing"] = function()
    local _, fake = today.opened()
    local notes = buffer.capture_notes()
    today.on("Today (1)")
    buffer.feed("<CR>")
    assert(notes[1] and notes[1].text:find("no task on this line", 1, true), vim.inspect(notes))
    assert(#fake.of("show") == 0)
  end,

  ["]] and [[ jump to the next and the previous header"] = function()
    today.opened()
    today.on("[ ] call the bank")
    buffer.feed("]]")
    assert(vim.api.nvim_win_get_cursor(0)[1] == 5, vim.api.nvim_win_get_cursor(0)[1])
    buffer.feed("[[[[")
    assert(vim.api.nvim_win_get_cursor(0)[1] == 1, vim.api.nvim_win_get_cursor(0)[1])
  end,

  ["<Tab> on a header closes its section into one line naming it and its count"] = function()
    today.opened()
    today.on("No date (3)")
    buffer.feed("<Tab>")
    assert(vim.fn.foldclosed(8) == 8 and vim.fn.foldclosedend(8) == 11)
    assert(grid.row(9):find("^  No date %(3%) ··") and vim.endswith(grid.row(9), "·"), grid.row(9))
    buffer.feed("<Tab>")
    assert(vim.fn.foldclosed(8) == -1)
  end,

  ["a closed parent shows its row and how many children dam counts open"] = function()
    today.opened()
    today.on("[ ] plan the move")
    buffer.feed("<Tab>")
    assert(vim.fn.foldclosed(9) == 9 and vim.fn.foldclosedend(9) == 10)
    assert(grid.row(10):find("^  %[ %] plan the move  %+2 open ··") and vim.endswith(grid.row(10), "·"), grid.row(9))
  end,

  ["a closed section stays closed when the list is read again"] = function()
    local _, fake = today.opened()
    today.on("Overdue (1)")
    buffer.feed("<Tab>r")
    buffer.wait_for(function()
      return #fake.of("list") == 8
    end)
    vim.wait(30)
    assert(vim.fn.foldclosed(1) == 1, vim.fn.foldclosed(1))
  end,

  ["<Plug>(dam-today) runs :Dam today"] = function()
    dofile(vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "../../plugin/damnit.lua"))
    assert(vim.fn.maparg("<Plug>(dam-today)", "n") == "<Cmd>Dam today<CR>", vim.fn.maparg("<Plug>(dam-today)", "n"))
  end,
}
