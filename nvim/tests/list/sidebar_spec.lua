local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local grid = require("helpers.grid")
local helper = require("helpers.app")

local function defined(options)
  local app, fake = helper.new(options)
  fake.reply("list", fixtures.answer("list-columns.json"))
  fake.reply("show", fixtures.answer("show.json"))
  local task = require("damnit.task").new(app)
  require("damnit.command").define(app, { require("damnit.list").new(app, task.open), task })
  vim.cmd.edit(vim.fn.tempname() .. ".txt")
  return vim.api.nvim_get_current_win(), fake, app
end

local function sidebar(command)
  vim.cmd(command or "Dam sidebar")
  buffer.wait_for(function()
    return vim.bo.filetype == "damlist" and #buffer.lines() > 1
  end)
  return vim.api.nvim_get_current_win()
end

local function plain_windows()
  return vim.tbl_filter(function(win)
    return vim.api.nvim_win_get_config(win).relative == ""
  end, vim.api.nvim_tabpage_list_wins(0))
end

return {
  [":Dam sidebar opens the list in a fixed 40 column split on the left"] = function()
    local main = defined()
    local side = sidebar()
    assert(side ~= main and #plain_windows() == 2)
    assert(vim.api.nvim_win_get_width(side) == 40, vim.api.nvim_win_get_width(side))
    assert(vim.wo[side].winfixwidth)
    assert(vim.fn.win_screenpos(side)[2] == 1, vim.inspect(vim.fn.win_screenpos(side)))
  end,

  [":Dam sidebar again closes it"] = function()
    local main = defined()
    sidebar()
    vim.cmd("Dam sidebar")
    assert(vim.deep_equal(plain_windows(), { main }), vim.inspect(plain_windows()))
  end,

  ["the sidebar option puts it on the right at its width"] = function()
    defined({ sidebar = { side = "right", width = 30 } })
    local side = sidebar()
    assert(vim.api.nvim_win_get_width(side) == 30)
    assert(vim.fn.win_screenpos(side)[2] == vim.o.columns - 29, vim.inspect(vim.fn.win_screenpos(side)))
  end,

  ["each tab has its own sidebar"] = function()
    defined()
    sidebar()
    vim.cmd.tabnew()
    local side = sidebar()
    assert(#plain_windows() == 2 and vim.api.nvim_win_get_width(side) == 40)
    vim.cmd("Dam sidebar")
    assert(#plain_windows() == 1)
    vim.cmd.tabclose()
    assert(#plain_windows() == 2)
  end,

  ["<CR> in the sidebar opens the task in the window it was opened from"] = function()
    local main = defined()
    local side = sidebar()
    buffer.feed("<CR>")
    assert(vim.api.nvim_get_current_win() == main)
    assert(vim.api.nvim_buf_get_name(0):find("^dam://task/"), vim.api.nvim_buf_get_name(0))
    assert(vim.bo[vim.api.nvim_win_get_buf(side)].filetype == "damlist")
    assert(vim.api.nvim_win_get_width(side) == 40)
  end,

  ["the sidebar's rows fit its width with the due at its right edge"] = function()
    defined()
    sidebar("Dam sidebar")
    vim.cmd.redraw()
    for row = 2, 5 do
      local text = grid.row(row, 40)
      assert(not text:find("#urgent", 1, true), text)
    end
    assert(grid.row(2, 40):find("2026%-09%-14$"), grid.row(2, 40))
  end,

  ["<Plug>(dam-sidebar) toggles the sidebar"] = function()
    dofile(vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "../../plugin/damnit.lua"))
    assert(
      vim.fn.maparg("<Plug>(dam-sidebar)", "n") == "<Cmd>Dam sidebar<CR>",
      vim.fn.maparg("<Plug>(dam-sidebar)", "n")
    )
  end,

  ["a sidebar left as the only window becomes a plain window and :Dam sidebar opens a new one"] = function()
    defined()
    local side = sidebar()
    vim.cmd.only()
    local again = sidebar()
    assert(again ~= side and #plain_windows() == 2, vim.inspect(plain_windows()))
    assert(not vim.wo[side].winfixwidth)
  end,

  ["<CR> in a sidebar left alone opens the task in a new split beside it"] = function()
    defined()
    local side = sidebar()
    vim.cmd.only()
    buffer.feed("<CR>")
    assert(#plain_windows() == 2 and vim.api.nvim_get_current_win() ~= side)
    assert(vim.api.nvim_buf_get_name(0):find("^dam://task/"), vim.api.nvim_buf_get_name(0))
    assert(vim.bo[vim.api.nvim_win_get_buf(side)].filetype == "damlist")
    assert(not vim.wo.winfixwidth)
  end,

  ["a sidebar and a main list of the same view keep their own buffers and columns, and both read again on a change"] = function()
    local main, fake, app = defined({ sidebar = { width = 24 } })
    fake.reply("list", fixtures.answer("list.json"))
    vim.cmd("Dam list")
    buffer.wait_for(function()
      return #buffer.lines() > 1
    end)
    local main_buf = vim.api.nvim_get_current_buf()
    local side = sidebar()
    assert(vim.api.nvim_buf_get_name(0) == "dam://sidebar/default", vim.api.nvim_buf_get_name(0))
    assert(vim.api.nvim_win_get_buf(main) == main_buf)
    vim.cmd.redraw({ bang = true })
    local main_row = grid.row(3):sub(vim.fn.byteidx(grid.row(3), 25) + 1)
    assert(main_row:find("context=errand", 1, true) and main_row:find("2026%-09%-25$"), main_row)
    assert(not grid.row(3, 24):find("errand", 1, true), grid.row(3, 24))
    fake.reply("list", fixtures.answer("list-columns.json"))
    app.say:changed(vim.fn.getcwd())
    buffer.wait_for(function()
      local side_lines = vim.api.nvim_buf_get_lines(vim.api.nvim_win_get_buf(side), 0, -1, false)
      local main_lines = vim.api.nvim_buf_get_lines(main_buf, 0, -1, false)
      return #side_lines == 6 and #main_lines == 6
    end)
  end,
}
