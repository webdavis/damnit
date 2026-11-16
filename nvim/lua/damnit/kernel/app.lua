local M = {}

---@param options damnit.Options
---@param process table
---@return damnit.App
function M.new(options, process)
  local say = require("damnit.kernel.say").new()
  local queue = require("damnit.kernel.queue").new()
  local screen = require("damnit.kernel.screen").new()
  return {
    options = options,
    say = say,
    queue = queue,
    screen = screen,
    keys = require("damnit.kernel.keys").new(screen),
    dam = require("damnit.kernel.dam").new(options, process, queue, say),
  }
end

return M
