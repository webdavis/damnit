local brief = require("damnit.handoff.brief")
local hosts_of = require("damnit.handoff.hosts")

local NOWHERE = "put the cursor on a task in a dam list, the status window or a task buffer"

local M = {}

---@param app damnit.App
---@param hosts? damnit.Hosts
---@return damnit.Feature
function M.new(app, hosts)
  hosts = hosts or hosts_of.made(app)

  local function sent(text, host, unavailable)
    local function fell_back(why)
      hosts.clipboard.send(text, function(_, said)
        app.say:warn(("%s: %s; %s"):format(host.name, why, said))
      end)
    end
    if unavailable then
      return fell_back(unavailable)
    end
    host.send(text, function(err, said)
      if err then
        return fell_back(err)
      end
      app.say:info(said)
    end)
  end

  ---@param oid string
  ---@param cwd string
  ---@param note string|nil
  local function hand_off(oid, cwd, note)
    app.dam:run({ args = { "show", oid }, cwd = cwd, key = "handoff" }, function(object, err)
      if err then
        return app.say:error(err)
      end
      local host, unavailable = hosts_of.chosen(app.options.handoff.host, hosts)
      sent(brief.text(object, note), host, unavailable)
    end)
  end

  local function asked(oid, cwd, note)
    if not oid then
      return app.say:warn(NOWHERE)
    end
    if note then
      return hand_off(oid, cwd, note)
    end
    vim.ui.input({ prompt = "note for the agent (empty for none): " }, function(typed)
      if typed then
        hand_off(oid, cwd, typed)
      end
    end)
  end

  app.keys:offer_task_key(function(at, group)
    return {
      lhs = "H",
      desc = "hand the task to an agent",
      group = group,
      mode = "n",
      run = function()
        local oid, cwd = at()
        if not oid then
          return app.say:warn("H acts on task rows; this line holds no task")
        end
        asked(oid, cwd, nil)
      end,
    }
  end)

  return {
    hand_off = hand_off,
    commands = {
      handoff = function(_, cmd)
        local note = cmd.args:gsub("^%s*%S+%s*", "", 1)
        local oid, cwd = app.keys:task_here()
        asked(oid, cwd, note ~= "" and note or nil)
      end,
    },
  }
end

return M
