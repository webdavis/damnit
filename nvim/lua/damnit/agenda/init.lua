local buffer = require("damnit.agenda.buffer")
local errors = require("damnit.kernel.error")

local M = {}

local UNITS = { "day", "week" }

---@param app damnit.App
---@param show fun(source: damnit.ListSource, cwd: string, smods?: table)
---@return damnit.Feature
function M.new(app, show)
  local agenda = buffer.new(show)
  return {
    commands = {
      agenda = function(args, cmd)
        local unit = args[1] or "week"
        if not vim.tbl_contains(UNITS, unit) then
          return app.say:error(errors.plugin(("%q is not an agenda window; use day or week"):format(unit)))
        end
        agenda.open(unit, 0, vim.fn.getcwd(), cmd.smods)
      end,
    },
    complete = {
      agenda = function(arglead)
        return vim.tbl_filter(function(unit)
          return vim.startswith(unit, arglead)
        end, UNITS)
      end,
    },
  }
end

return M
