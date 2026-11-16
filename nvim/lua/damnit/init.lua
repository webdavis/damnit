local M = {}

local running
local statusline

---@param opts? table
---@return damnit.App
function M.setup(opts)
  local options = require("damnit.kernel.options").resolve(opts or {})
  require("damnit.kernel.highlights").setup()
  if running then
    running.queue:cancel()
  end
  running = require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
  require("damnit.kernel.refresh").new(running.say)
  local handoff = require("damnit.handoff").new(running)
  local task = require("damnit.task").new(running)
  statusline = require("damnit.statusline").new(running)
  require("damnit.command").define(running, {
    require("damnit.messages").new(running),
    require("damnit.list").new(running, task.open),
    task,
    require("damnit.status").new(running, task.open),
    require("damnit.capture").new(running),
    handoff,
  })
  return running
end

---@return damnit.App
function M.app()
  if not running then
    M.setup({})
  end
  return running
end

---@return damnit.Statusline
local function statusline_feature()
  if not statusline then
    M.setup({})
  end
  return statusline
end

---@param opts? { highlight?: boolean }
---@return string
function M.statusline(opts)
  return statusline_feature().component(opts)
end

---@return damnit.Counts|nil
function M.counts()
  return statusline_feature().counts()
end

return M
