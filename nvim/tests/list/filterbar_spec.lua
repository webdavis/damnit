local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local filterbar = require("helpers.filterbar")
local prompt = require("helpers.prompt")

local EXIT_USAGE = 2

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["f opens dam list at the top of the list window, prefilled with the view's conditions as flags"] = function()
    filterbar.opened()
    local list_win = vim.api.nvim_get_current_win()
    filterbar.bar()
    local config = vim.api.nvim_win_get_config(prompt.window())
    assert(vim.trim(config.title[1][1]) == "dam list", vim.inspect(config.title))
    assert(config.relative == "win" and config.win == list_win and config.row == 0, vim.inspect(config))
    assert(prompt.line() == "--due ..today+7 --labels bug", prompt.line())
  end,

  ["each pause runs dam list with the bar's flags in the list's folder and the list redraws under the bar"] = function()
    local _, fake = filterbar.opened()
    local list_buf = vim.api.nvim_get_current_buf()
    filterbar.bar("<End> --overdue")
    prompt.footer_reads("1 row")
    local last = filterbar.lists(fake)[#fake.of("list")]
    same(last, { "list", "--due", "..today+7", "--labels", "bug", "--overdue" })
    assert(fake.of("list")[1].cwd == fake.of("list")[#fake.of("list")].cwd)
    same(buffer.lines(list_buf), { "[ ] p2 buy oat milk" })
  end,

  ["a refusal shows dam's message, keeps the last good rows, and offers the refused flag's shapes"] = function()
    local _, fake = filterbar.opened()
    local list_buf = vim.api.nvim_get_current_buf()
    local rows = buffer.lines(list_buf)
    fake.reply("list", fixtures.refusal("error-usage-selection.json", EXIT_USAGE))
    filterbar.bar("<End> --priority urgent")
    local message = fixtures.decoded("error-usage-selection.json").error.message
    prompt.below_reads(message)
    same(buffer.lines(list_buf), rows)
    local bar_buf = vim.api.nvim_win_get_buf(prompt.window())
    local line = prompt.line() .. " --priority x"
    vim.api.nvim_buf_set_lines(bar_buf, 0, -1, false, { line })
    vim.api.nvim_win_set_cursor(prompt.window(), { 1, #line - 1 })
    local complete = vim.b[bar_buf].damnit_complete
    complete(1)
    local words = vim.tbl_map(function(candidate)
      return candidate.word
    end, complete(0))
    same(words, fixtures.decoded("error-usage-selection.json").error.selection.shapes)
  end,

  ["<CR> keeps the conditions: the bar closes, the winbar names the view with a star, and r reads them again"] = function()
    local _, fake = filterbar.opened()
    filterbar.bar("<End> --overdue")
    prompt.footer_reads("1 row")
    prompt.more("<CR>")
    assert(prompt.window() == nil, "the bar is still open")
    assert(vim.wo.winbar:find("week*", 1, true), vim.wo.winbar)
    local count = #fake.of("list")
    buffer.feed("r")
    filterbar.wait_lists(fake, count + 1)
    same(filterbar.lists(fake)[count + 1], { "list", "--due", "..today+7", "--labels", "bug", "--overdue" })
    same(buffer.lines(), { "[ ] p2 buy oat milk" })
  end,

  ["<Esc> restores the view as it was"] = function()
    local _, fake = filterbar.opened()
    local rows = buffer.lines()
    filterbar.bar("<End> --overdue")
    prompt.footer_reads("1 row")
    prompt.type("<Esc>")
    assert(prompt.window() == nil, "the bar is still open")
    buffer.wait_for(function()
      return vim.deep_equal(buffer.lines(), rows)
    end)
    local last = fake.of("list")[#fake.of("list")]
    same(vim.list_slice(last.argv, 2), { "list", "--request", "-", "-o", "json" })
    assert(not vim.wo.winbar:find("week*", 1, true), vim.wo.winbar)
  end,

  ["a sectioned view opens the bar empty and shows one list while it is open, as its footer says"] = function()
    local _, fake = filterbar.opened({}, "Dam today")
    local list_buf = vim.api.nvim_get_current_buf()
    filterbar.bar()
    assert(prompt.line() == "", prompt.line())
    prompt.footer_reads("3 rows · one list in place of the sections")
    same(filterbar.lists(fake)[#fake.of("list")], { "list" })
    same(buffer.lines(list_buf), { "[ ] call mom", "[ ] p2 buy oat milk", "  [ ] pack the bag" })
  end,

  ["a saved filter's view opens the bar with the filter's conditions"] = function()
    filterbar.opened({}, "Dam list today")
    filterbar.bar()
    assert(prompt.line() == "--due ..today", prompt.line())
  end,

  [":Dam filter opens the bar on the list in the window, and outside a list says to open one"] = function()
    filterbar.opened()
    vim.cmd("Dam filter")
    assert(prompt.window() and prompt.line() == "--due ..today+7 --labels bug")
    prompt.type("<Esc>")
    local notes = buffer.capture_notes()
    vim.cmd.enew()
    vim.cmd("Dam filter")
    assert(notes[1] and notes[1].text:find(":Dam list", 1, true), vim.inspect(notes))
  end,
}
