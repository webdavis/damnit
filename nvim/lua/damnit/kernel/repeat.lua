local M = {}

local OPERATOR = "v:lua.require'damnit.kernel.repeat'.operator"

---@class damnit.Repeat
---@field rearm fun() makes `.` repeat this key again after typing elsewhere, such as in a prompt

---@param first fun() runs now
---@param again fun(count: integer) runs on each `.`, with the count typed before it or 0
---@return damnit.Repeat
function M.start(first, again)
  local started, skip = false, false
  local function run()
    if skip then
      skip = false
    elseif started then
      again(vim.v.count)
    else
      started = true
      first()
    end
  end
  vim.g.damnit_repeat = run
  vim.o.operatorfunc = OPERATOR
  vim.api.nvim_feedkeys("g@l", "ni", false)
  return {
    rearm = function()
      vim.schedule(function()
        skip = true
        vim.g.damnit_repeat = run
        vim.o.operatorfunc = OPERATOR
        vim.api.nvim_feedkeys("g@l", "n", false)
      end)
    end,
  }
end

function M.operator()
  vim.g.damnit_repeat()
end

return M
