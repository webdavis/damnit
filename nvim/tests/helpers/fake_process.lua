local fixtures = require("helpers.fixtures")

local M = {}

local KILLED = { code = 0, signal = 15, stdout = "", stderr = "" }

local function verb_of(argv)
  return argv[2]
end

---@return table
function M.new()
  local fake = {
    calls = {},
    replies = {
      ["--version"] = { code = 0, stdout = fixtures.read("version.txt"), stderr = "" },
      config = fixtures.answer("config-list.json"),
    },
  }

  function fake.reply(verb, reply)
    fake.replies[verb] = reply
  end

  function fake.of(verb)
    return vim.tbl_filter(function(call)
      return verb_of(call.argv) == verb
    end, fake.calls)
  end

  function fake.release(call, reply)
    vim.schedule(function()
      call.on_exit(reply or call.reply)
    end)
  end

  fake.port = {
    run = function(argv, opts, on_exit)
      local reply = assert(fake.replies[verb_of(argv)], "the fake has no reply for " .. table.concat(argv, " "))
      local call = { argv = argv, cwd = opts.cwd, stdin = opts.stdin, reply = reply, on_exit = on_exit, killed = false }
      table.insert(fake.calls, call)
      if not reply.hold then
        fake.release(call)
      end
      return {
        kill = function()
          call.killed = true
          fake.release(call, KILLED)
        end,
      }
    end,
  }

  return fake
end

return M
