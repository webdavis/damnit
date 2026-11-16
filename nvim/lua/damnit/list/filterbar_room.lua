local M = {}

local BORDERS = 2

local function namespace()
  return vim.api.nvim_create_namespace("damnit.filterbar")
end

---@param win integer
---@param fill integer
local function view(win, fill)
  vim.api.nvim_win_call(win, function()
    vim.fn.winrestview({ topline = 1, topfill = fill })
  end)
end

---@param buf integer the list's buffer
---@param win integer the list's window
---@param bar integer the bar's window, over the list window's top
function M.make(buf, win, bar)
  vim.api.nvim_buf_clear_namespace(buf, namespace(), 0, -1)
  if not (vim.api.nvim_win_is_valid(win) and vim.api.nvim_win_is_valid(bar)) then
    return
  end
  local height = vim.api.nvim_win_get_config(bar).height + BORDERS
  local lines = {}
  for _ = 1, height do
    table.insert(lines, { { "" } })
  end
  vim.api.nvim_buf_set_extmark(buf, namespace(), 0, 0, { virt_lines = lines, virt_lines_above = true })
  view(win, height)
end

---@param buf integer
---@param win integer
function M.clear(buf, win)
  vim.api.nvim_buf_clear_namespace(buf, namespace(), 0, -1)
  if vim.api.nvim_win_is_valid(win) then
    view(win, 0)
  end
end

return M
