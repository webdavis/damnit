local errors = require("damnit.kernel.error")
local version = require("damnit.kernel.version")

local M = {}

---@return string|nil version, damnit.Error|nil err
local function read(exit)
  if exit.code ~= 0 then
    return nil, errors.of_exit(exit.code, exit.stderr)
  end
  local found = version.of(exit.stdout)
  if not found then
    return nil, errors.plugin(("dam --version printed %q, which names no version"):format(vim.trim(exit.stdout)))
  end
  if not version.supported(found) then
    return found, errors.plugin(version.refusal(found))
  end
  return found, nil
end

---@param spawn damnit.Spawn
function M.new(spawn)
  local settled, waiting = nil, nil
  local handshake = {}

  function handshake.version()
    return settled and settled.version
  end

  ---@param cwd string
  ---@param ready fun(err: damnit.Error|nil)
  function handshake.wait(cwd, ready)
    if settled then
      return ready(settled.err)
    end
    if waiting then
      return table.insert(waiting, ready)
    end
    waiting = { ready }
    spawn({ "--version" }, { cwd = cwd }, function(exit, spawn_err)
      local found, err = nil, spawn_err
      if not err then
        found, err = read(exit)
      end
      if found then
        settled = { version = found, err = err }
      end
      local everyone = waiting
      waiting = nil
      for _, each in ipairs(everyone) do
        each(err)
      end
    end)
  end

  return handshake
end

return M
