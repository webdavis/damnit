local M = {}

---@class damnit.ProgressRecord
---@field step string
---@field stage "started"|"finished"|"failed"
---@field done integer
---@field total? integer
---@field seconds integer
---@field remote? string

local STATUS = { started = "running", finished = "success", failed = "failed" }
local ENDED = { success = "finished", failed = "failed", cancel = "cancelled" }

---@param step string
---@param status string
---@return string
local function text_of(step, status)
  return ENDED[status] and (step .. " " .. ENDED[status]) or step
end

---@param line string
---@return damnit.ProgressRecord|nil
function M.record(line)
  local ok, decoded = pcall(vim.json.decode, line, { luanil = { object = true, array = true } })
  if ok and type(decoded) == "table" and type(decoded.step) == "string" and STATUS[decoded.stage] then
    return decoded
  end
end

---@param record damnit.ProgressRecord
---@return string text, vim.api.keyset.echo_opts opts
function M.message(record)
  local total = record.total or 0
  local status = STATUS[record.stage]
  return text_of(record.step, status),
    {
      kind = "progress",
      status = status,
      percent = total > 0 and math.floor(record.done * 100 / total) or nil,
      title = record.remote,
      source = "dam",
    }
end

---@class damnit.ProgressShown
---@field line fun(line: string)
---@field stop fun(status: "failed"|"cancel")

---@param text string
---@param opts vim.api.keyset.echo_opts
---@return integer|string
local function echo(text, opts)
  return vim.api.nvim_echo({ { text } }, opts.status ~= "running", opts)
end

---@param cancel fun()
---@return damnit.ProgressShown
function M.shown(cancel)
  local running = nil
  local function settle()
    local was = running
    running = nil
    if was then
      pcall(vim.api.nvim_del_autocmd, was.watch)
    end
    return was
  end
  local function watch(id)
    return vim.api.nvim_create_autocmd("Progress", {
      pattern = "dam",
      callback = function(ev)
        if ev.data.id == id and ev.data.status == "cancel" then
          settle()
          cancel()
        end
      end,
    })
  end
  local shown = {}
  function shown.line(line)
    local record = M.record(line)
    if not record then
      return
    end
    local text, opts = M.message(record)
    local was = settle()
    opts.id = was and was.opts.id
    opts.id = echo(text, opts)
    if opts.status == "running" then
      running = { step = record.step, opts = opts, watch = watch(opts.id) }
    end
  end
  function shown.stop(status)
    local was = settle()
    if was then
      was.opts.status = status
      echo(text_of(was.step, status), was.opts)
    end
  end
  return shown
end

return M
