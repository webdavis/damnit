local M = {}

---@param first fun() runs now
---@param again fun(count: integer) runs on each `.`, with the count typed before it or 0
function M.start(first, again)
  local started = false
  vim.g.damnit_repeat = function()
    if started then
      return again(vim.v.count)
    end
    started = true
    first()
  end
  vim.o.operatorfunc = "v:lua.require'damnit.kernel.repeat'.operator"
  vim.api.nvim_feedkeys("g@l", "ni", false)
end

function M.operator()
  vim.g.damnit_repeat()
end

return M
