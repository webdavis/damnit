local float = require("damnit.kernel.float")

local M = {}

---@class damnit.ConfirmSpec
---@field title string
---@field lines string[]
---@field marks damnit.Mark[]
---@field yes fun()

---@param screen damnit.Screen
---@param spec damnit.ConfirmSpec
function M.open(screen, spec)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].filetype = "damconfirm"
  screen:draw(buf, spec.lines, spec.marks)
  local win = float.sheet(buf, spec.title, "y yes · n no")
  local function close()
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end
  local opts = { buffer = buf, nowait = true }
  for _, lhs in ipairs({ "y", "<CR>" }) do
    vim.keymap.set("n", lhs, function()
      close()
      spec.yes()
    end, opts)
  end
  for _, lhs in ipairs({ "n", "<Esc>", "q" }) do
    vim.keymap.set("n", lhs, close, opts)
  end
  vim.api.nvim_create_autocmd("WinLeave", { buffer = buf, once = true, callback = close })
end

return M
