local M = {}

---@class damnit.Exit
---@field code integer
---@field signal? integer
---@field stdout string
---@field stderr string
---@field missing? boolean

---@class damnit.Process
---@field kill fun()

---@class damnit.ProcessOptions
---@field cwd string
---@field stdin? string
---@field on_line? fun(line: string)

---@param on_line fun(line: string)
---@return fun(err: string|nil, data: string|nil) read, fun(): string whole
local function stderr_lines(on_line)
  local chunks, pending = {}, ""
  local function read(_, data)
    if not data then
      return
    end
    table.insert(chunks, data)
    local parts = vim.split(pending .. data, "\n", { plain = true })
    pending = table.remove(parts)
    for _, line in ipairs(parts) do
      vim.schedule(function()
        on_line(line)
      end)
    end
  end
  local function whole()
    if pending ~= "" then
      local last = pending
      pending = ""
      vim.schedule(function()
        on_line(last)
      end)
    end
    return table.concat(chunks)
  end
  return read, whole
end

---@param argv string[]
---@param opts damnit.ProcessOptions
---@param on_exit fun(exit: damnit.Exit)
---@return damnit.Process
function M.run(argv, opts, on_exit)
  local read, whole = nil, nil
  if opts.on_line then
    read, whole = stderr_lines(opts.on_line)
  end
  local system_opts = { cwd = opts.cwd, stdin = opts.stdin, text = true, stderr = read }
  local started, system = pcall(vim.system, argv, system_opts, function(out)
    local stderr = whole and whole() or out.stderr or ""
    vim.schedule(function()
      on_exit({ code = out.code, signal = out.signal, stdout = out.stdout or "", stderr = stderr })
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
