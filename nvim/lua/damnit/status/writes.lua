local M = {}

local function pushed(answer)
  return "pushed: "
    .. table.concat(
      vim.tbl_map(function(remote)
        return ("%s sent %d, failed %d"):format(remote.remote, remote.sent, #remote.failed)
      end, answer.remotes),
      "; "
    )
end

local function pulled(answer)
  return "pulled: "
    .. table.concat(
      vim.tbl_map(function(remote)
        return ("%s created %d, updated %d, conflicts %d"):format(
          remote.remote,
          remote.created,
          remote.updated,
          remote.conflicts
        )
      end, answer.remotes),
      "; "
    )
end

---@class damnit.StatusWrites
---@field send fun(args: string[], cwd: string, done?: fun(answer: table|nil))
---@field commit fun(cwd: string, message?: string)
---@field push fun(cwd: string, remote?: string)
---@field pull fun(cwd: string, remote?: string)

---@param app damnit.App
---@return damnit.StatusWrites
function M.new(app)
  local writes = {}

  function writes.send(args, cwd, done)
    app.dam:write({ args = args, cwd = cwd }, function(answer)
      if answer and done then
        done(answer)
      end
    end)
  end

  function writes.commit(cwd, message)
    local function commit(text)
      if text and vim.trim(text) ~= "" then
        writes.send({ "commit", "-m", text }, cwd)
      end
    end
    if message and message ~= "" then
      return commit(message)
    end
    vim.ui.input({ prompt = "commit message: " }, commit)
  end

  local function remote_call(verb, summary)
    return function(cwd, remote)
      app.dam:write({ args = { verb, remote }, cwd = cwd, progress = true }, function(answer)
        if answer then
          app.say:info(summary(answer))
        end
      end)
    end
  end

  writes.push = remote_call("push", pushed)
  writes.pull = remote_call("pull", pulled)
  return writes
end

return M
