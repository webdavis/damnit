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
      group = "every window",
      run = function()
        local list = self.shown[buf]
        local oid = self:object_oid(buf)
        if not oid then
          return self.app.say:warn("no task on this line to open")
        end
        self.open_task(oid, list.call.cwd)
      end,
    },
    {
      lhs = "r",
      desc = "read the list again",
      group = "every window",
      run = function()
        self:read(buf)
      end,
    },
  }
  vim.list_extend(specs, section_keys())
  vim.list_extend(specs, self:change_keys(buf))
  vim.list_extend(
    specs,
    self.app.keys:for_task(buf, "task rows", function()
      return self:object_oid(buf), self.shown[buf].call.cwd
    end)
  )
  vim.list_extend(specs, self:filter_keys(buf))
  vim.list_extend(specs, self:source_keys(buf))
  self.app.keys:bind(buf, vim.list_extend(specs, self.more_keys and self.more_keys(buf) or {}))
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = self.group,
    buffer = buf,
    callback = function()
      self.shown[buf] = nil
    end,
  })
end

---@param buf integer
---@return string|nil oid the object on the cursor's row, an event or a readonly row too
function Bindings:object_oid(buf)
  local row = (self.shown[buf].arranged or {})[vim.api.nvim_win_get_cursor(0)[1]]
  return row and row.object and row.object.oid or nil
end

---@param buf integer
---@return damnit.KeySpec[]
function Bindings:source_keys(buf)
  local keys = self.shown[buf].source.keys
  return keys and keys(self:window(buf)) or {}
end

return Bindings
