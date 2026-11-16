local ui = require("helpers.ui")

local SESSION, OWN, AGENT = "session:1", "block:own", "block:agent"

local function recording(replies)
  local port = { calls = {} }
  function port.run(argv, opts, on_exit)
    table.insert(port.calls, { argv = argv, stdin = opts.stdin })
    on_exit(table.remove(replies, 1) or { code = 0, stdout = "", stderr = "" })
  end
  return port
end

local function in_rex(run)
  local saved = { REX_SESSION = vim.env.REX_SESSION, REX_BLOCK = vim.env.REX_BLOCK, PATH = vim.env.PATH }
  vim.env.REX_SESSION, vim.env.REX_BLOCK = SESSION, OWN
  local ok, err = xpcall(run, debug.traceback)
  for name, value in pairs(saved) do
    vim.env[name] = value
  end
  assert(ok, err)
end

local function sent(port, answers)
  local result = {}
  in_rex(function()
    ui.answering(answers, function(prompts)
      require("damnit.handoff.rex").new(port).send("fix it\nthen test", function(err, said)
        result = { err = err, said = said, prompts = prompts }
      end)
    end)
  end)
  return result
end

local function listing()
  return { code = 0, stdout = OWN .. "\n" .. AGENT .. "\n", stderr = "" }
end

local function folder_with_rex()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.fn.writefile({ "#!/bin/sh", "exit 0" }, vim.fs.joinpath(dir, "rex"))
  vim.fn.setfperm(vim.fs.joinpath(dir, "rex"), "rwxr-xr-x")
  return dir
end

return {
  ["rex is available inside a Rex pane with rex on PATH"] = function()
    in_rex(function()
      vim.env.PATH = folder_with_rex()
      local available, why = require("damnit.handoff.rex").new(recording({})).available()
      assert(available and why == nil, tostring(why))
    end)
  end,

  ["rex is not available outside a Rex pane"] = function()
    in_rex(function()
      vim.env.PATH, vim.env.REX_BLOCK = folder_with_rex(), nil
      local available, why = require("damnit.handoff.rex").new(recording({})).available()
      assert(not available and why == "REX_BLOCK is not set, so Neovim is not running in Rex", tostring(why))
    end)
  end,

  ["rex is not available without rex on PATH"] = function()
    in_rex(function()
      vim.env.PATH = vim.fn.tempname()
      local available, why = require("damnit.handoff.rex").new(recording({})).available()
      assert(not available and why == "rex is not on PATH", tostring(why))
    end)
  end,

  ["the brief goes to the chosen pane as one paste, then enter"] = function()
    local port = recording({ listing() })
    local result = sent(port, { select = AGENT })
    assert(result.err == nil and result.said == "handed to rex: " .. AGENT, vim.inspect(result))
    assert(
      result.prompts[1] == "hand the task to which Rex pane? (ids from `rex block ls`)",
      vim.inspect(result.prompts)
    )
    local argvs = vim.tbl_map(function(call)
      return call.argv
    end, port.calls)
    assert(
      vim.deep_equal(argvs, {
        { "rex", "block", "ls", "--autostart=false", "-s", SESSION, "-q", "--short=false" },
        { "rex", "send", "--autostart=false", "-s", SESSION, "-b", AGENT },
        { "rex", "send-key", "--autostart=false", "-s", SESSION, "-b", AGENT, "enter" },
      }),
      vim.inspect(argvs)
    )
    assert(port.calls[2].stdin == "\27[200~fix it\nthen test\27[201~", vim.inspect(port.calls[2].stdin))
  end,

  ["the person's own pane is not offered"] = function()
    local offered
    in_rex(function()
      local select = vim.ui.select
      vim.ui.select = function(items)
        offered = items
      end
      require("damnit.handoff.rex").new(recording({ listing() })).send("brief", function() end)
      vim.ui.select = select
    end)
    assert(vim.deep_equal(offered, { AGENT }), vim.inspect(offered))
  end,

  ["no pane chosen sends nothing"] = function()
    local port = recording({ listing() })
    local result = sent(port, {})
    assert(result.err == "no pane chosen" and #port.calls == 1, vim.inspect(result))
  end,

  ["rex's refusal is the reason given"] = function()
    local port = recording({ listing(), { code = 1, stdout = "", stderr = "no block named block:agent\n" } })
    local result = sent(port, { select = AGENT })
    assert(result.err == "no block named block:agent" and #port.calls == 2, vim.inspect(result))
  end,
}
