local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local grid = require("helpers.grid")
local helper = require("helpers.app")

local EXIT_REFUSED = 4

local function floats()
  return vim.tbl_filter(function(win)
    return vim.api.nvim_win_get_config(win).relative ~= ""
  end, vim.api.nvim_list_wins())
end

local function peeking(reply)
  local app, fake = helper.new()
  fake.reply("show", reply or fixtures.answer("show-related.json"))
  local opened = {}
  local peek = require("damnit.peek").new(app, function(oid, cwd)
    table.insert(opened, { oid = oid, cwd = cwd })
  end)
  vim.api.nvim_buf_set_lines(0, 0, -1, false, { "one", "two", "three" })
  vim.cmd.redraw()
  return peek, fake, opened
end

local function peeked(peek)
  local before = floats()[1]
  peek.peek("abcdef1", "/tmp")
  buffer.wait_for(function()
    return #floats() == 1 and floats()[1] ~= before
  end)
  return floats()[1]
end

local function footer_text(win)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, vim.api.nvim_win_get_config(win).footer))
end

return {
  ["peek reads dam show in the folder given and opens a card at the cursor without entering it"] = function()
    local peek, fake = peeking()
    local source = vim.api.nvim_get_current_win()
    local win = peeked(peek)
    assert(vim.deep_equal(fake.of("show")[1].argv, { "dam", "show", "abcdef1", "-o", "json" }))
    assert(fake.of("show")[1].cwd == "/tmp", fake.of("show")[1].cwd)
    assert(vim.api.nvim_get_current_win() == source, "the card took the cursor")
    local config = vim.api.nvim_win_get_config(win)
    assert(config.relative == "win" and config.row == 1, vim.inspect(config))
    assert(config.title[1][1] == " plan the garden ", vim.inspect(config.title))
    assert(footer_text(win) == " <id-1> · overdue ", footer_text(win))
    assert(config.width == 24, config.width)
    local lines = vim.api.nvim_buf_get_lines(vim.api.nvim_win_get_buf(win), 0, -1, false)
    assert(lines[1] == "due         2026-09-25" and lines[#lines] == "… 8 more lines", vim.inspect(lines))
  end,

  ["moving the cursor in the source window closes the card"] = function()
    local peek = peeking()
    peeked(peek)
    vim.api.nvim_win_set_cursor(0, { 2, 0 })
    vim.api.nvim_exec_autocmds("CursorMoved", {})
    assert(#floats() == 0, "the card stayed")
  end,

  ["K again enters the card, and q leaves it closed and back in the source"] = function()
    local peek = peeking()
    local source = vim.api.nvim_get_current_win()
    local win = peeked(peek)
    peek.again()
    assert(vim.api.nvim_get_current_win() == win, "K did not enter the card")
    buffer.feed("q")
    assert(#floats() == 0 and vim.api.nvim_get_current_win() == source)
  end,

  ["<Esc> in the card closes it"] = function()
    local peek = peeking()
    peeked(peek)
    peek.again()
    buffer.feed("<Esc>")
    assert(#floats() == 0)
  end,

  ["<CR> in the card opens the task it shows"] = function()
    local peek, _, opened = peeking()
    peeked(peek)
    peek.again()
    buffer.feed("<CR>")
    assert(#floats() == 0)
    assert(vim.deep_equal(opened, { { oid = "abcdef1", cwd = "/tmp" } }), vim.inspect(opened))
  end,

  ["a second peek replaces the first"] = function()
    local peek = peeking()
    peeked(peek)
    peeked(peek)
    assert(#floats() == 1, #floats())
  end,

  ["a refused read shows dam's message and opens nothing"] = function()
    local notes = buffer.capture_notes()
    local peek = peeking(fixtures.refusal("error-refused.json", EXIT_REFUSED))
    peek.peek("0000000", "/tmp")
    buffer.wait_for(function()
      return #notes > 0
    end)
    vim.wait(30)
    assert(#notes == 1 and notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
    assert(#floats() == 0)
  end,

  ["the card is drawn under the cursor with its title in the top border"] = function()
    local peek = peeking()
    peeked(peek)
    vim.cmd.redraw({ bang = true })
    assert(grid.row(1):find("^one"), grid.row(1))
    local top = grid.row(2)
    assert(top:find("╭ plan the garden ─", 1, true), top)
    assert(grid.row(3):find("│due         2026%-09%-25"), grid.row(3))
  end,

  ["a tall card under a winbar stops above the status line"] = function()
    local peek = peeking()
    vim.wo.winbar = "bar"
    vim.cmd.redraw()
    peeked(peek)
    vim.cmd.redraw({ bang = true })
    vim.cmd.redraw({ bang = true })
    assert(grid.row(22):find("^╰"), grid.row(22))
    assert(not grid.row(23):find("╰", 1, true), grid.row(23))
  end,
}
