local M = {}

---@param row integer
---@param width? integer
---@return string
function M.row(row, width)
  vim.cmd.redraw()
  local cells = {}
  for col = 1, width or vim.o.columns do
    table.insert(cells, vim.fn.screenstring(row, col))
  end
  return table.concat(cells)
end

return M
