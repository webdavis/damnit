local M = {}

local running

---@param opts? table
---@return damnit.App
function M.setup(opts)
  local options = require("damnit.kernel.options").resolve(opts or {})
  if running then
    running.queue:cancel()
  end
  running = require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
  require("damnit.command").define(running, {
    require("damnit.messages").new(running),
  })
  return running
end

return M
