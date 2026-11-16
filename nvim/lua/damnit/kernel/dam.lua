local errors = require("damnit.kernel.error")
local handshake = require("damnit.kernel.handshake")
local progress = require("damnit.kernel.progress")
local spawn = require("damnit.kernel.spawn")

local Dam = {}
Dam.__index = Dam

local M = {}

---@param options damnit.Options
---@param process { run: fun(argv: string[], opts: table, on_exit: fun(exit: damnit.Exit)): damnit.Process }
---@param queue damnit.Queue
---@param say damnit.Say
---@return damnit.Dam
function M.new(options, process, queue, say)
  local spawned = spawn.new(options, process)
  return setmetatable({ spawn = spawned, handshake = handshake.new(spawned), queue = queue, say = say, reads = 0 }, Dam)
end

local function answer_of(exit, err, report)
  if err then
    return nil, err
  end
  if exit.code ~= 0 and not (report and exit.code == 1) then
    return nil, errors.of_exit(exit.code, exit.stderr)
  end
  local ok, decoded = pcall(vim.json.decode, exit.stdout, { luanil = { object = true, array = true } })
  if ok and type(decoded) == "table" then
    return decoded, nil
  end
  return nil, errors.not_json(exit.stderr)
end

local function args_of(call, shown)
  local args = vim.list_extend({}, call.args)
  if shown then
    table.insert(args, "--progress")
  end
  return vim.list_extend(args, { "-o", "json" })
end

function Dam:call(call, done, shown)
  local cancelled, spawned = false, nil
  self.handshake.wait(call.cwd, function(err)
    if cancelled then
      return
    end
    if err then
      return done(nil, err)
    end
    local opts = { cwd = call.cwd, stdin = call.stdin, on_line = shown and shown.line }
    spawned = self.spawn(args_of(call, shown), opts, function(exit, spawn_err)
      if shown then
        shown.stop("failed")
      end
      done(answer_of(exit, spawn_err, call.report))
    end)
  end)
  return {
    cancel = function()
      cancelled = true
      if spawned then
        spawned.cancel()
      end
      if shown then
        shown.stop("cancel")
      end
    end,
  }
end

function Dam:queued(call, mode, lane, done)
  local state = { cancelled = false }
  local handle = {}
  local shown = call.progress and progress.shown(function()
    handle.cancel()
  end) or nil
  self.queue:submit(lane, function(finish)
    state.finish = finish
    if state.cancelled then
      finish()
      return { cancel = function() end }
    end
    state.inner = self:call(call, function(answer, err)
      finish()
      done(answer, err)
    end, shown)
    return state.inner
  end, mode)
  handle.cancel = function()
    state.cancelled = true
    if state.inner then
      state.inner.cancel()
    end
    if state.finish then
      state.finish()
    end
  end
  return handle
end

function Dam:run(call, done)
  self.reads = self.reads + 1
  return self:queued(call, "latest", call.key or ("read " .. self.reads), done)
end

function Dam:write(call, done)
  return self:queued(call, "ordered", call.key or "writes", function(answer, err)
    if err and call.placed then
      self.say:keep(err)
    elseif err then
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
