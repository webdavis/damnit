local errors = require("damnit.kernel.error")

local M = {}

---@alias damnit.Spawn fun(args: string[], opts: damnit.ProcessOptions, on_exit: fun(exit: damnit.Exit, err: damnit.Error|nil)): damnit.Handle

---@param options damnit.Options
---@param process { run: fun(argv: string[], opts: table, on_exit: fun(exit: damnit.Exit)): damnit.Process }
---@return damnit.Spawn
function M.new(options, process)
  return function(args, opts, on_exit)
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
    local argv = vim.list_extend({ options.dam }, args)
    local on_line = opts.on_line
      and function(line)
        if not settled then
          opts.on_line(line)
        end
      end
    local process_opts = { cwd = opts.cwd, stdin = opts.stdin, on_line = on_line }
    local running = process.run(argv, process_opts, function(exit)
      if settle() then
        on_exit(exit, exit.missing and errors.missing(options.dam) or nil)
      end
    end)
    local ms = options.timeout_ms
    timer:start(
      ms,
      0,
      vim.schedule_wrap(function()
        if settle() then
          running.kill()
          on_exit({}, errors.plugin(("dam %s took longer than %d ms and was stopped"):format(args[1], ms)))
        end
      end)
    )
    return {
      cancel = function()
        if settle() then
          running.kill()
        end
      end,
    }
  end
end

return M
