local buffer = require("damnit.status.buffer")
local writes_of = require("damnit.status.writes")

local M = {}

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
---@param open_log? fun(cwd: string, smods?: table, at?: string)
---@return damnit.Feature
function M.new(app, open_task, open_log)
  local writes = writes_of.new(app)
  local status = buffer.new(app, writes, open_task, open_log)
  local function open(_, cmd)
    status:open(vim.fn.getcwd(), cmd.smods)
  end
  return {
    open = function(cwd, smods)
      status:open(cwd, smods)
    end,
    default = open,
    commands = {
      status = open,
      commit = function(args)
        writes.commit(vim.fn.getcwd(), table.concat(args, " "))
      end,
      push = function(args)
        writes.push(vim.fn.getcwd(), args[1])
      end,
      pull = function(args)
        writes.pull(vim.fn.getcwd(), args[1])
      end,
    },
  }
end

return M
