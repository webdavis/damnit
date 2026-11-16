local diagnostics = require("damnit.kernel.diagnostics")

local Refusals = {}

---@param err damnit.Error
function Refusals:refused(err)
  local buf = self.buf
  if not (buf and vim.api.nvim_buf_is_valid(buf)) then
    return
  end
  local oids = {}
  for line, target in pairs(self.targets) do
    oids[line] = target.oid or false
  end
  diagnostics.show(buf, err, diagnostics.rows_of(err, oids))
end

---@param argv string[]
function Refusals:send(argv)
  local buf = self.buf
  self.writes.send(argv, self.cwd, nil, {
    show = function(err)
      self:refused(err)
    end,
    clear = function()
      if buf and vim.api.nvim_buf_is_valid(buf) then
        diagnostics.clear(buf)
      end
    end,
  })
end

return Refusals
