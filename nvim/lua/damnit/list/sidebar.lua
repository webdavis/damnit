local M = {}

---@param win integer
---@return boolean
local function alone(win)
  local plain = vim.tbl_filter(function(other)
    return vim.api.nvim_win_get_config(other).relative == ""
  end, vim.api.nvim_tabpage_list_wins(vim.api.nvim_win_get_tabpage(win)))
  return #plain == 1
end

---@param app damnit.App
---@param pick damnit.Pick
---@return fun(name: string|nil, cwd: string)
function M.toggle(app, pick)
  return function(name, cwd)
    local open = vim.t.damnit_sidebar
    vim.t.damnit_sidebar = nil
    if open and vim.api.nvim_win_is_valid(open) then
      if not alone(open) then
        return vim.api.nvim_win_close(open, false)
      end
      vim.w[open].damnit_side = nil
      vim.wo[open].winfixwidth = false
    end
    local side = app.options.sidebar
    local split = side.side == "left" and "topleft" or "botright"
    pick.open(name, cwd, { vertical = true, split = split }, "dam://sidebar/")
    local win = vim.api.nvim_get_current_win()
    vim.api.nvim_win_set_width(win, side.width)
    vim.wo[win].winfixwidth = true
    vim.w[win].damnit_side = true
    vim.t.damnit_sidebar = win
  end
end

return M
