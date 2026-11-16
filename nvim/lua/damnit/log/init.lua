local buffer = require("damnit.log.buffer")

local M = {}

---@param app damnit.App
---@param open_status fun(cwd: string)
---@return damnit.Feature
function M.new(app, open_status)
  local log = buffer.new(app, open_status)
  local function open(cwd, smods, at)
    log:open(cwd, smods, at)
  end
  app.keys:everywhere({
    lhs = "gL",
    desc = "open the log",
    run = function()
      open(vim.fn.getcwd())
    end,
  })
  return {
    open = open,
    commands = {
      log = function(_, cmd)
        open(vim.fn.getcwd(), cmd.smods)
      end,
    },
  }
end

return M
