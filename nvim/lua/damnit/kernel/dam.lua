local errors = require("damnit.kernel.error")
local handshake = require("damnit.kernel.handshake")

local Dam = {}
Dam.__index = Dam

local M = {}

---@param options damnit.Options
---@param process { run: fun(argv: string[], opts: table, on_exit: fun(exit: damnit.Exit)): damnit.Process }
---@param queue damnit.Queue
---@param say damnit.Say
---@return damnit.Dam
function M.new(options, process, queue, say)
  local dam = setmetatable({ options = options, process = process, queue = queue, say = say, reads = 0 }, Dam)
  dam.handshake = handshake.new(function(...)
    return dam:spawn(...)
  end)
  return dam
end

local function answer_of(exit, err)
  if err then
    return nil, err
  end
  if exit.code ~= 0 then
    return nil, errors.of_exit(exit.code, exit.stderr)
  end
  local ok, decoded = pcall(vim.json.decode, exit.stdout, { luanil = { object = true, array = true } })
  if ok and type(decoded) == "table" then
    return decoded, nil
  end
  return nil, errors.not_json(exit.stderr)
end

function Dam:spawn(args, cwd, stdin, on_exit)
  local timer = assert(vim.uv.new_timer())
  local settled = false
  local function settle()
    if settled then
      return false
    end
    settled = true
    timer:stop()
    timer:close()
    return true
  end
  local argv = vim.list_extend({ self.options.dam }, args)
  local process = self.process.run(argv, { cwd = cwd, stdin = stdin }, function(exit)
    if settle() then
      on_exit(exit, exit.missing and errors.missing(self.options.dam) or nil)
    end
  end)
  local ms = self.options.timeout_ms
  timer:start(
    ms,
    0,
    vim.schedule_wrap(function()
      if settle() then
        process.kill()
        on_exit({}, errors.plugin(("dam %s took longer than %d ms and was stopped"):format(args[1], ms)))
      end
    end)
  )
  return {
    cancel = function()
      if settle() then
        process.kill()
      end
    end,
  }
end

function Dam:call(call, done)
  local cancelled, spawned = false, nil
  self.handshake.wait(call.cwd, function(err)
    if cancelled then
      return
    end
    if err then
      return done(nil, err)
    end
    local args = vim.list_extend(vim.list_extend({}, call.args), { "-o", "json" })
    spawned = self:spawn(args, call.cwd, call.stdin, function(exit, spawn_err)
      done(answer_of(exit, spawn_err))
    end)
  end)
  return {
    cancel = function()
      cancelled = true
      if spawned then
        spawned.cancel()
      end
    end,
  }
end

function Dam:queued(call, mode, lane, done)
  local state = { cancelled = false }
  self.queue:submit(lane, function(finish)
    state.finish = finish
    if state.cancelled then
      finish()
      return { cancel = function() end }
    end
    state.inner = self:call(call, function(answer, err)
      finish()
      done(answer, err)
    end)
    return state.inner
  end, mode)
  return {
    cancel = function()
      state.cancelled = true
      if state.inner then
        state.inner.cancel()
      end
      if state.finish then
        state.finish()
      end
    end,
  }
end

function Dam:run(call, done)
  self.reads = self.reads + 1
  return self:queued(call, "latest", call.key or ("read " .. self.reads), done)
end

function Dam:write(call, done)
  return self:queued(call, "ordered", call.key or "writes", function(answer, err)
    if err then
      self.say:error(err)
    else
      self.say:changed(call.cwd)
    end
    done(answer, err)
  end)
end

function Dam:version()
  return self.handshake.version()
end

return M
