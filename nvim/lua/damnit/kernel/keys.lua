local Keys = {}
Keys.__index = Keys

local M = {}

---@param screen damnit.Screen
---@return damnit.Keys
function M.new(screen)
  return setmetatable({ screen = screen }, Keys)
end

---@param specs damnit.KeySpec[]
---@return string[]
function M.help_lines(specs)
  return vim.tbl_map(function(spec)
    return spec.lhs .. "  " .. spec.desc
  end, specs)
end

function Keys:help(specs)
  local lines = M.help_lines(specs)
  local buf = self.screen:scratch("dam://keys", "damkeys")
  self.screen:draw(buf, lines, {})
  local width = math.max(unpack(vim.tbl_map(vim.fn.strdisplaywidth, lines)))
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    row = 1,
    col = math.max(vim.o.columns - width - 4, 0),
    width = width,
    height = #lines,
    style = "minimal",
    border = "rounded",
    title = " keys ",
  })
  local function close()
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end
  for _, lhs in ipairs({ "q", "<Esc>" }) do
    vim.keymap.set("n", lhs, close, { buffer = buf, nowait = true })
  end
  vim.api.nvim_create_autocmd("WinLeave", { buffer = buf, once = true, callback = close })
end

---@param buf integer
---@param specs damnit.KeySpec[]
function Keys:bind(buf, specs)
  local all = vim.list_slice(specs)
  table.insert(all, {
    lhs = "g?",
    desc = "list these keys",
    run = function()
      self:help(all)
    end,
  })
  for _, spec in ipairs(all) do
    vim.keymap.set(spec.mode or "n", spec.lhs, spec.run, { buffer = buf, desc = spec.desc, nowait = true })
  end
end

return M
