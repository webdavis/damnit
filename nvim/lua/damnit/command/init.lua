local errors = require("damnit.kernel.error")

local M = {}

local function merged(features)
  local commands, complete, default = {}, {}, nil
  for _, feature in ipairs(features) do
    commands = vim.tbl_extend("error", commands, feature.commands)
    complete = vim.tbl_extend("error", complete, feature.complete or {})
    default = default or feature.default
  end
  return commands, complete, default
end

local function cancel(app)
  return function()
    local stopped = app.queue:cancel()
    if stopped == 0 then
      return app.say:info("no dam call is running")
    end
    app.say:info(("cancelled %d dam call%s"):format(stopped, stopped == 1 and "" or "s"))
  end
end

local function completer(names, complete)
  return function(arglead, line)
    local words = vim.split(line, "%s+")
    if #words <= 2 then
      return vim.tbl_filter(function(name)
        return vim.startswith(name, arglead)
      end, names)
    end
    local more = complete[words[2]]
    return more and more(arglead) or {}
  end
end

---@param app damnit.App
---@param features damnit.Feature[]
function M.define(app, features)
  local commands, complete, default = merged(features)
  commands.cancel = cancel(app)
  local names = vim.tbl_keys(commands)
  table.sort(names)
  local valid = table.concat(names, ", ")
  vim.api.nvim_create_user_command("Dam", function(cmd)
    local name = cmd.fargs[1]
    if not name and default then
      return default({}, cmd)
    end
    if not name then
      return app.say:info("dam subcommands: " .. valid)
    end
    if not commands[name] then
      return app.say:error(errors.plugin(("%q is not a :Dam subcommand; use one of %s"):format(name, valid)))
    end
    commands[name](vim.list_slice(cmd.fargs, 2), cmd)
  end, { nargs = "*", range = true, desc = "dam", complete = completer(names, complete) })
end

return M
