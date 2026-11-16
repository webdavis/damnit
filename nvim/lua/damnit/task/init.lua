local buffer = require("damnit.task.buffer")
local errors = require("damnit.kernel.error")

local M = {}

local function source_folder()
  local name = vim.api.nvim_buf_get_name(0)
  if vim.bo.buftype == "" and name ~= "" then
    return vim.fs.dirname(name)
  end
  return vim.fn.getcwd()
end

---@param app damnit.App
---@return damnit.Feature
function M.new(app)
  local tasks = buffer.new(app)
  local function open(oid, cwd, smods)
    tasks:open(oid, cwd, smods)
  end
  return {
    open = open,
    commands = {
      task = function(args, cmd)
        if not args[1] then
          return app.say:error(errors.plugin("name the task: :Dam task <oid>"))
        end
        open(args[1], source_folder(), cmd.smods)
      end,
    },
  }
end

return M
