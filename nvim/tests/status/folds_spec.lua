local buffer = require("helpers.buffer")
local grid = require("helpers.grid")
local status = require("helpers.status")

local function header_rows()
  local rows = {}
  for n, line in ipairs(buffer.lines()) do
    if line:find("^%u%l+ %(%d+%)$") then
      table.insert(rows, n)
    end
  end
  return rows
end

return {
  ["]] and [[ jump between the status window's section headers"] = function()
    status.opened()
    local headers = header_rows()
    vim.api.nvim_win_set_cursor(0, { headers[1] + 1, 0 })
    buffer.feed("]]")
    assert(vim.api.nvim_win_get_cursor(0)[1] == headers[2], vim.inspect({ headers, vim.api.nvim_win_get_cursor(0) }))
    buffer.feed("]][[")
    assert(vim.api.nvim_win_get_cursor(0)[1] == headers[2], vim.inspect({ headers, vim.api.nvim_win_get_cursor(0) }))
  end,

  ["<Tab> on a section header closes the section into its header line"] = function()
    status.opened()
    local headers = header_rows()
    vim.api.nvim_win_set_cursor(0, { headers[2], 0 })
    local title = buffer.lines()[headers[2]]
    buffer.feed("<Tab>")
    assert(vim.fn.foldclosed(headers[2]) == headers[2], vim.fn.foldclosed(headers[2]))
    assert(vim.fn.foldclosedend(headers[2]) == headers[3] - 2, vim.fn.foldclosedend(headers[2]))
    local row = grid.row(headers[2] + 1)
    assert(vim.startswith(row, title .. " ··") and vim.endswith(row, "·"), row)
  end,
}
