local buffer = require("helpers.buffer")
local grid = require("helpers.grid")
local log = require("helpers.log")
local status = require("helpers.status")

local ERRAND = "<id-2>  <now>  plan the errand  1 change   buy oat milk"

return {
  [":Dam log opens dam://log with one row per commit dam log lists"] = function()
    local _, fake = log.opened()
    assert(vim.api.nvim_buf_get_name(0) == "dam://log", vim.api.nvim_buf_get_name(0))
    assert(vim.deep_equal(fake.of("log")[1].argv, { "dam", "log", "-o", "json" }), vim.inspect(fake.of("log")))
    assert(buffer.lines()[2] == ERRAND, vim.inspect(buffer.lines()))
  end,

  ["the log window's winbar counts the commits"] = function()
    log.opened()
    assert(vim.startswith(grid.row(1), " log · 3 commits"), grid.row(1))
  end,

  ["<CR> shows a commit's changes from dam show under it, and <CR> again hides them"] = function()
    local _, fake = log.opened()
    status.on(ERRAND)
    buffer.feed("<CR>")
    buffer.wait_for(function()
      return log.virtual_lines()[2] ~= nil
    end)
    assert(vim.deep_equal(fake.of("show")[1].argv, { "dam", "show", "<id-2>", "-o", "json" }))
    local shown = log.virtual_lines()[2]
    assert(vim.deep_equal(shown, { "    update  buy oat milk  labels, priority, due" }), vim.inspect(shown))
    buffer.feed("<CR>")
    assert(vim.tbl_isempty(log.virtual_lines()), vim.inspect(log.virtual_lines()))
  end,

  ["gL in the status window opens the log"] = function()
    local _, fake = log.defined()
    status.open(fake)
    buffer.feed("gL")
    buffer.wait_for(function()
      return vim.bo.filetype == "damlog" and #buffer.lines() == 3
    end)
  end,

  ["<CR> on a Recent row in the status window opens the log at that commit"] = function()
    local _, fake = log.defined()
    status.open(fake)
    buffer.wait_for(function()
      return vim.tbl_contains(buffer.lines(), "Recent (3)")
    end)
    local row = vim.fn.index(buffer.lines(), "Recent (3)") + 3
    vim.api.nvim_win_set_cursor(0, { row, 0 })
    buffer.feed("<CR>")
    buffer.wait_for(function()
      return vim.bo.filetype == "damlog" and vim.api.nvim_get_current_line() == ERRAND
    end)
  end,
}
