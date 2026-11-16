local M = {}

local QUIET_MS = 1000

---@param say damnit.Say
---@param now? fun(): integer
function M.new(say, now)
  now = now or vim.uv.now
  local last
  vim.api.nvim_create_autocmd("FocusGained", {
    group = vim.api.nvim_create_augroup("damnit.refresh", { clear = true }),
    callback = function()
      local at = now()
      if last and at - last < QUIET_MS then
        return
      end
      last = at
      say:changed(vim.fn.getcwd())
    end,
  })
end

return M
