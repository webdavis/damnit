local view = require("damnit.list.view")

local M = {}

---@param lists table
---@param buf integer
---@return damnit.Landing
local function landing(lists, buf)
  return function(oid, done)
    lists:land_on(buf, oid, done)
  end
end

---@param app damnit.App
---@param quickadd damnit.QuickAdd
---@param lists fun(): table
---@return fun(buf: integer): damnit.KeySpec[]
function M.keys(app, quickadd, lists)
  return function(buf)
    local function open(after_of)
      return function()
        local context = lists():context(buf)
        local after = after_of(context)
        if after then
          quickadd.open(context.cwd, after, landing(lists(), buf))
        end
      end
    end
    return {
      {
        lhs = "a",
        desc = "new task",
        run = open(function(context)
          local path = view.single_path(context.view, app.options.views)
          return path and (" --path " .. path) or ""
        end),
      },
      {
        lhs = "o",
        desc = "new task under this one",
        run = open(function(context)
          if not context.oid then
            return app.say:warn("no task on this line to put the new one under")
          end
          return " --parent " .. context.oid
        end),
      },
    }
  end
end

---@param quickadd damnit.QuickAdd
---@param lists table
---@return fun(args: string[], cmd: table)
function M.command(quickadd, lists)
  return function(_, cmd)
    local buf = vim.api.nvim_get_current_buf()
    local context = lists:context(buf)
    local cwd = context and context.cwd or vim.fn.getcwd()
    local land = context and landing(lists, buf) or nil
    local text = cmd.args:gsub("^%s*%S+%s*", "", 1)
    if vim.trim(text) == "" then
      return quickadd.open(cwd, "", land)
    end
    quickadd.send(text, cwd, land)
  end
end

return M
