local errors = require("damnit.kernel.error")

local M = {}

---@param log damnit.Error[]
---@return string[]
function M.lines(log)
  if #log == 0 then
    return { "no dam errors yet" }
  end
  local lines = {}
  for n, err in ipairs(log) do
    if n > 1 then
      table.insert(lines, "")
    end
    local said = vim.split(err.message, "\n")
    said[1] = ("%s%s: %s"):format(err.kind, err.rule and (" " .. err.rule) or "", said[1])
    vim.list_extend(lines, said)
    local command = errors.suggested(err)
    if command then
      table.insert(lines, "run: " .. command)
    end
  end
  return lines
end

---@param app damnit.App
---@return damnit.Feature
function M.new(app)
  return {
    commands = {
      messages = function(_, cmd)
        local buf = app.screen:scratch("dam://messages", "dammessages")
        app.screen:draw(buf, M.lines(app.say:log()), {})
        app.screen:show(buf, cmd.smods)
      end,
    },
  }
end

return M
