local M = {}

---@class damnit.Exit
---@field code integer
---@field signal? integer
---@field stdout string
---@field stderr string
---@field missing? boolean

---@class damnit.Process
---@field kill fun()

---@param argv string[]
---@param opts { cwd: string, stdin?: string }
---@param on_exit fun(exit: damnit.Exit)
---@return damnit.Process
function M.run(argv, opts, on_exit)
  local started, system = pcall(vim.system, argv, { cwd = opts.cwd, stdin = opts.stdin, text = true }, function(out)
    vim.schedule(function()
      on_exit({ code = out.code, signal = out.signal, stdout = out.stdout or "", stderr = out.stderr or "" })
    end)
  end)
  if not started then
    vim.schedule(function()
      on_exit({ code = -1, stdout = "", stderr = tostring(system), missing = true })
    end)
    return { kill = function() end }
  end
  return {
    kill = function()
      if not system:is_closing() then
        system:kill("sigterm")
      end
    end,
  }
end

return M
