local folds = require("damnit.kernel.folds")

local Bindings = {}

---@return damnit.KeySpec[]
local function section_keys()
  return folds.keys()
end

---@param buf integer
function Bindings:bind(buf)
  local specs = {
    {
      lhs = "<CR>",
      desc = "open the task",
      run = function()
        local list = self.shown[buf]
        local oid = list.oids[vim.api.nvim_win_get_cursor(0)[1]]
        if not oid then
          return self.app.say:warn("no task on this line to open")
        end
        self.open_task(oid, list.call.cwd)
      end,
    },
    {
      lhs = "r",
      desc = "read the list again",
      run = function()
        self:read(buf)
      end,
    },
  }
  vim.list_extend(specs, section_keys())
  vim.list_extend(specs, self:change_keys(buf))
  self.app.keys:bind(buf, vim.list_extend(specs, self.more_keys and self.more_keys(buf) or {}))
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.shown[buf] = nil
    end,
  })
end

return Bindings
