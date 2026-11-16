local PASTE_START, PASTE_END = "\27[200~", "\27[201~"

local M = {}

---@param process { run: fun(argv: string[], opts: damnit.ProcessOptions, on_exit: fun(exit: damnit.Exit)) }
---@return damnit.Host
function M.new(process)
  local function scoped(words, extra)
    local argv = vim.list_extend({ "rex" }, words)
    table.insert(argv, "--autostart=false")
    if vim.env.REX_SESSION then
      vim.list_extend(argv, { "-s", vim.env.REX_SESSION })
    end
    return vim.list_extend(argv, extra)
  end

  local function rex(argv, stdin, done)
    process.run(argv, { cwd = vim.fn.getcwd(), stdin = stdin }, function(exit)
      if exit.code == 0 then
        return done(nil, exit.stdout)
      end
      local said = vim.trim(exit.stderr)
      done(said ~= "" and said or ("%s exited %d"):format(table.concat(argv, " ", 1, 2), exit.code))
    end)
  end

  local function typed(block, brief, done)
    rex(scoped({ "send" }, { "-b", block }), PASTE_START .. brief .. PASTE_END, function(err)
      if err then
        return done(err)
      end
      rex(scoped({ "send-key" }, { "-b", block, "enter" }), nil, function(key_err)
        done(key_err, not key_err and ("handed to rex: " .. block) or nil)
      end)
    end)
  end

  return {
    name = "rex",
    available = function()
      if not vim.env.REX_BLOCK then
        return false, "REX_BLOCK is not set, so Neovim is not running in Rex"
      end
      if vim.fn.executable("rex") == 0 then
        return false, "rex is not on PATH"
      end
      return true
    end,
    send = function(brief, done)
      rex(scoped({ "block", "ls" }, { "-q", "--short=false" }), nil, function(err, listed)
        if err then
          return done(err)
        end
        local blocks = vim.tbl_filter(function(block)
          return block ~= vim.env.REX_BLOCK
        end, vim.split(listed, "\n", { trimempty = true }))
        if #blocks == 0 then
          return done("no other pane in this Rex session")
        end
        vim.ui.select(blocks, { prompt = "hand the task to which Rex pane? (ids from `rex block ls`)" }, function(block)
          if not block then
            return done("no pane chosen")
          end
          typed(block, brief, done)
        end)
      end)
    end,
  }
end

return M
