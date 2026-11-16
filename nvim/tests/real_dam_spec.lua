local helper = require("helpers.app")
local wait_for = require("helpers.buffer").wait_for

local tests_dir = vim.fs.dirname(debug.getinfo(1, "S").source:sub(2))
local dam_bin =
  vim.fs.normalize(vim.fs.abspath(vim.env.DAM_BIN or vim.fs.joinpath(tests_dir, "../../target/debug/dam")))

local SANDBOXED = { "DAM_CONFIG", "DAM_STORE", "HOME" }

local function in_sandbox(run)
  local sandbox = vim.fn.tempname()
  vim.fn.mkdir(sandbox, "p")
  local saved = {}
  for _, name in ipairs(SANDBOXED) do
    saved[name] = vim.env[name]
  end
  vim.env.DAM_CONFIG = vim.fs.joinpath(sandbox, "config", "dam", "config.lua")
  vim.env.DAM_STORE = vim.fs.joinpath(sandbox, "dam.db")
  vim.env.HOME = sandbox
  local ok, err = pcall(run, sandbox)
  for _, name in ipairs(SANDBOXED) do
    vim.env[name] = saved[name]
  end
  assert(ok, err)
end

local function real_app()
  local options = require("damnit.kernel.options").resolve({ dam = dam_bin })
  return require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
end

return {
  ["the client runs the real dam list and reads an empty list"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(function(sandbox)
      local got = helper.call(real_app(), { args = { "list" }, cwd = sandbox })
      wait_for(function()
        return got.done
      end, 20000)
      assert(not got.err, vim.inspect(got.err))
      assert(vim.deep_equal(got.answer.objects, {}), vim.inspect(got.answer))
    end)
  end,
}
